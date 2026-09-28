use std::borrow::Cow;

use atomic_write_file::AtomicWriteFile;
use concat_string::concat_string;
use ffmpeg_next::{
    Error as AvError, Packet as AvPacket, codec as avcodec, encoder as avencoder,
    filter as avfilter, format as avformat, frame as avframe, media as avmedia, util as avutil,
};
use loole::Receiver;
use tracing::instrument;

use super::{Path, Sink};
use crate::{Error, config, error};

struct Input {
    context: avformat::context::Input,
    decoder: avcodec::decoder::Audio,
    index: usize,
}

struct Output {
    context: avformat::context::Output,
    encoder: avcodec::encoder::Audio,
    in_time_base: avutil::rational::Rational,
    out_time_base: avutil::rational::Rational,
}

struct Graph {
    graph: avfilter::Graph,
    spec: Cow<'static, str>,
}

struct Filter {
    source: avfilter::Context,
    sink: avfilter::Context,
}

pub struct Transcoder {
    input: Input,
    output: Output,
    graph: Graph,
}

impl Input {
    fn new(input: &str) -> Result<Self, Error> {
        let context = avformat::input(input)?;

        let stream = context
            .streams()
            .best(avmedia::Type::Audio)
            .ok_or_else(|| error::Kind::MissingAudioTrack)?;
        let index = stream.index();

        let mut decoder =
            avcodec::context::Context::from_parameters(stream.parameters())?.decoder().audio()?;
        decoder.set_parameters(stream.parameters())?;
        decoder.set_packet_time_base(stream.time_base());

        Ok(Self { context, decoder, index })
    }
}

impl Output {
    fn new(sink: Sink, bitrate: u32, decoder: &avcodec::decoder::Audio) -> Result<Self, Error> {
        let filename = sink.filename();
        let mut context =
            ffmpeg_next::format::output_to_stream(sink.try_into()?, Some(filename), None)?;

        if cfg!(test) {
            // Set bitexact for deterministic transcoding output.
            unsafe {
                // AVFMT_FLAG_BITEXACT
                (*context.as_mut_ptr()).flags |= 1024;
            }
        }

        let codec = avencoder::find(context.format().codec(filename, avmedia::Type::Audio))
            .ok_or_else(|| error::Kind::MissingEncoder)?
            .audio()?;

        // bit to kbit
        let bitrate = (bitrate * 1000).try_into()?;
        // Opus sample rate will always be 48000Hz.
        let sample_rate =
            if codec.id() == avcodec::Id::OPUS { 48000 } else { decoder.rate().try_into()? };

        let mut encoder = avcodec::context::Context::new_with_codec(*codec).encoder().audio()?;
        encoder.set_channel_layout(decoder.channel_layout());
        encoder.set_format(
            codec
                .formats()
                .as_mut()
                .and_then(Iterator::next)
                .ok_or_else(|| error::Kind::MissingEncoderSampleFmts)?,
        );
        encoder.set_rate(sample_rate);
        encoder.set_bit_rate(bitrate);
        encoder.set_max_bit_rate(bitrate);
        encoder.set_time_base((1, sample_rate));

        if context.format().flags().contains(avformat::Flags::GLOBAL_HEADER) {
            encoder.set_flags(avcodec::Flags::GLOBAL_HEADER);
        }

        let encoder = encoder.open()?;
        {
            let mut stream = context.add_stream(codec)?;
            stream.set_parameters(&encoder);
            stream.set_time_base(encoder.time_base());
        }
        context.write_header()?;

        let in_time_base = decoder.time_base();
        let out_time_base = encoder.time_base();

        Ok(Self { context, encoder, in_time_base, out_time_base })
    }

    fn encode(&mut self, frame: Option<&avframe::Audio>) -> Result<(), Error> {
        if let Some(frame) = frame {
            self.encoder.send_frame(frame)
        } else {
            self.encoder.send_eof()
        }?;

        let mut packet = AvPacket::empty();
        loop {
            match self.encoder.receive_packet(&mut packet) {
                Err(AvError::Other { errno: avutil::error::EAGAIN } | AvError::Eof) => {
                    break Ok(());
                }
                Err(error) => {
                    break Err(error.into());
                }
                Ok(()) => {
                    packet.set_stream(0);
                    packet.rescale_ts(self.in_time_base, self.out_time_base);
                    packet.write_interleaved(&mut self.context)?;
                }
            }
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        if let Some(codec) = self.encoder.codec()
            && codec.capabilities().contains(avcodec::Capabilities::DELAY)
        {
            self.encode(None)
        } else {
            Ok(())
        }
    }
}

impl Graph {
    fn new(
        decoder: &avcodec::decoder::Audio,
        encoder: &avcodec::encoder::Audio,
        offset: u32,
    ) -> Self {
        let mut specs: Vec<Cow<'static, str>> = vec![];
        if offset > 0 {
            specs.push(concat_string!("atrim=start=", offset.to_string()).into());
        }
        if decoder.rate() != encoder.rate() {
            specs.push("aresample=resampler=soxr".into());
        }

        let spec = if specs.is_empty() { "anull".into() } else { specs.join(",").into() };

        Self { graph: avfilter::Graph::new(), spec }
    }
}

impl Filter {
    pub fn new(
        graph: &mut Graph,
        decoder: &avcodec::decoder::Audio,
        encoder: &avcodec::encoder::Audio,
    ) -> Result<Self, Error> {
        let source_ref =
            avfilter::find("abuffer").ok_or_else(|| error::Kind::MissingAVFilter("abuffer"))?;
        let sink_ref = avfilter::find("abuffersink")
            .ok_or_else(|| error::Kind::MissingAVFilter("abuffersink"))?;

        let source_arg = concat_string!(
            "time_base=",
            &decoder.packet_time_base().to_string(),
            ":sample_rate=",
            &decoder.rate().to_string(),
            ":sample_fmt=",
            &decoder.format().name(),
            ":channel_layout=0x",
            faster_hex::hex_string(&decoder.channel_layout().bits().to_be_bytes())
        );
        tracing::debug!(?source_arg);
        let source = graph.graph.add(&source_ref, "in", &source_arg)?;

        let sink_arg = concat_string!(
            "samplerates=",
            encoder.rate().to_string(),
            ":sample_formats=",
            encoder.format().name(),
            ":channel_layouts=0x",
            faster_hex::hex_string(&encoder.channel_layout().bits().to_be_bytes())
        );
        tracing::debug!(?sink_arg);
        let mut sink = graph.graph.add(&sink_ref, "out", &sink_arg)?;
        if let Some(codec) = encoder.codec()
            && !codec.capabilities().contains(avcodec::Capabilities::VARIABLE_FRAME_SIZE)
        {
            sink.sink().set_frame_size(encoder.frame_size());
        }

        // Yes. The output name is in.
        graph.graph.output("in", 0)?.input("out", 0)?.parse(&graph.spec)?;
        graph.graph.validate()?;
        tracing::debug!(graph = graph.graph.dump());

        Ok(Self { source, sink })
    }

    fn filter_and_encode(
        &mut self,
        output: &mut Output,
        frame: Option<&avframe::Audio>,
    ) -> Result<(), Error> {
        let mut source = self.source.source();
        let mut sink = self.sink.sink();

        if let Some(frame) = frame { source.add(frame) } else { source.flush() }?;

        let mut frame = avframe::Audio::empty();
        loop {
            match sink.frame(&mut frame) {
                Err(AvError::Other { errno: avutil::error::EAGAIN } | AvError::Eof) => {
                    break Ok(());
                }
                Err(error) => {
                    break Err(error.into());
                }
                Ok(()) => {
                    output.encode(Some(&frame))?;
                }
            }
        }
    }
}

impl Transcoder {
    #[cfg_attr(not(coverage_nightly), instrument)]
    pub fn spawn(
        config: &config::Transcode,
        path: Path,
        format: nghe_api::common::format::Transcode,
        bitrate: u32,
        offset: u32,
    ) -> (Receiver<Vec<u8>>, tokio::task::JoinHandle<Result<(), Error>>) {
        let (tx, rx) = crate::sync::channel(config.channel_size);
        let buffer_size = config.buffer_size;

        let span = tracing::Span::current();
        let handle = tokio::task::spawn_blocking(move || {
            let _entered = span.enter();

            let atomic_cache = path.cache.map(AtomicWriteFile::open).transpose()?;
            let cache = atomic_cache.as_ref().map(|file| file.as_file().try_clone()).transpose()?;
            let sink = Sink { tx, buffer_size, format, cache };

            let mut transcoder = Self::new(&path.input, sink, bitrate, offset)?;
            transcoder.transcode()?;
            atomic_cache.map(AtomicWriteFile::commit).transpose()?;
            Ok(())
        });

        (rx, handle)
    }

    fn new(input: &str, sink: Sink, bitrate: u32, offset: u32) -> Result<Self, Error> {
        let input = Input::new(input)?;
        let output = Output::new(sink, bitrate, &input.decoder)?;
        let graph = Graph::new(&input.decoder, &output.encoder, offset);
        Ok(Self { input, output, graph })
    }

    #[cfg_attr(
        not(coverage_nightly),
        instrument(skip_all, ret(level = "debug"), err(Debug, level = "debug"))
    )]
    pub fn transcode(&mut self) -> Result<(), Error> {
        let mut filter = Filter::new(&mut self.graph, &self.input.decoder, &self.output.encoder)?;

        for (stream, packet) in self.input.context.packets() {
            // Ignore non audio stream packets.
            if stream.index() != self.input.index {
                continue;
            }
            self.input.decoder.send_packet(&packet)?;

            let mut frame = avframe::Audio::empty();
            loop {
                match self.input.decoder.receive_frame(&mut frame) {
                    Err(AvError::Other { errno: avutil::error::EAGAIN } | AvError::Eof) => {
                        break;
                    }
                    Err(error) => {
                        return Err(error.into());
                    }
                    Ok(()) => {
                        filter.filter_and_encode(&mut self.output, Some(&frame))?;
                    }
                }
            }
        }

        // Flush the filter graph by pushing none packet to its source.
        filter.filter_and_encode(&mut self.output, None)?;

        self.output.flush()?;
        self.output.context.write_trailer()?;

        Ok(())
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use futures_lite::{StreamExt, stream};
    use nghe_api::common::format;

    use super::*;
    use crate::config;

    impl Transcoder {
        pub async fn spawn_collect(
            config: &config::Transcode,
            input: impl Into<String>,
            format: format::Transcode,
            bitrate: u32,
            offset: u32,
        ) -> Vec<u8> {
            let (rx, handle) = Transcoder::spawn(
                config,
                Path { input: input.into(), cache: None },
                format,
                bitrate,
                offset,
            );
            let data = rx.into_stream().map(stream::iter).flatten().collect().await;
            handle.await.unwrap().unwrap();
            data
        }
    }
}

#[cfg(all(test, hearing_test))]
#[coverage(off)]
mod tests {
    use nghe_api::common::format;
    use rstest::rstest;
    use typed_path::Utf8PlatformPath;

    use super::*;
    use crate::config;

    #[rstest]
    #[case(format::Transcode::Opus, 64)]
    #[case(format::Transcode::Mp3, 320)]
    #[tokio::test]
    async fn test_hearing(
        #[case] format: format::Transcode,
        #[case] bitrate: u32,
        #[values(0, 10)] offset: u32,
    ) {
        ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Trace);
        let input = env!("NGHE_HEARING_TEST_INPUT");
        let config = config::Transcode::default();
        let data = Transcoder::spawn_collect(&config, input, format, bitrate, offset).await;

        tokio::fs::write(
            Utf8PlatformPath::new(env!("NGHE_HEARING_TEST_OUTPUT"))
                .join(concat_string!(bitrate.to_string(), "-", offset.to_string()))
                .with_extension(format.as_ref()),
            &data,
        )
        .await
        .unwrap();
    }
}
