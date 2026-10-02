use std::borrow::Cow;

use atomic_write_file::AtomicWriteFile;
use concat_string::concat_string;
use ffmpeg_next::{
    Error as AvError, Packet as AvPacket, Rescale as _, codec as avcodec, encoder as avencoder,
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
    codec: avcodec::Audio,
    encoder: avcodec::encoder::Audio,
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
    fn new(sink: Sink, bit_rate: u32, decoder: &avcodec::decoder::Audio) -> Result<Self, Error> {
        let filename = sink.filename();
        let mut context =
            ffmpeg_next::format::output_to_stream(sink.try_into()?, Some(filename), None)?;

        if cfg!(test) {
            // Set bitexact for deterministic transcoding output.
            unsafe {
                (*context.as_mut_ptr()).flags |= ffmpeg_next::ffi::AVFMT_FLAG_BITEXACT;
            }
        }

        let codec = avencoder::find(context.format().codec(filename, avmedia::Type::Audio))
            .ok_or_else(|| error::Kind::MissingEncoderCodec)?
            .audio()?;

        // bit to kbit
        let bit_rate = (bit_rate * 1000).try_into()?;

        // Choose a sample rate that is closest to the decoder's one
        let decoder_rate = decoder.rate().try_into()?;
        let encoder_rate = codec.rates().map_or(decoder_rate, |iter| {
            iter.min_by_key(|rate| rate.abs_diff(decoder_rate)).unwrap_or(decoder_rate)
        });

        // Choose a sample format that is closest to the decoder's one
        let decoder_format = decoder.format();
        let encoder_format = codec.formats().map_or(decoder_format, |iter| {
            iter.max_by_key(|format| {
                10 * i32::from(format.bytes() == decoder_format.bytes())
                    + i32::from(format.is_packed() == decoder_format.is_packed())
            })
            .unwrap_or(decoder_format)
        });

        let mut encoder = avcodec::context::Context::new_with_codec(*codec).encoder().audio()?;

        encoder.set_channel_layout(decoder.channel_layout());
        encoder.set_bit_rate(bit_rate);
        encoder.set_max_bit_rate(bit_rate);
        encoder.set_rate(encoder_rate);
        encoder.set_time_base((1, encoder_rate));
        encoder.set_format(encoder_format);

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

        Ok(Self { context, codec, encoder })
    }

    fn encode(&mut self, frame: Option<&mut avframe::Audio>) -> Result<(), Error> {
        if let Some(frame) = frame {
            if let Some(pts) = frame.pts() {
                let frame_timebase = unsafe { (*frame.as_ptr()).time_base };
                frame.set_pts(Some(pts.rescale(frame_timebase, self.encoder.time_base())));
            };
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
                    packet.write_interleaved(&mut self.context)?;
                }
            }
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        if self.codec.capabilities().contains(avcodec::Capabilities::DELAY) {
            self.encode(None)
        } else {
            Ok(())
        }
    }
}

impl Graph {
    fn new(input: &Input, output: &Output, offset: u32) -> Self {
        let mut specs: Vec<Cow<'static, str>> = vec![];
        if offset > 0 {
            specs.push(concat_string!("atrim=start=", offset.to_string()).into());
        }
        if input.decoder.rate() != output.encoder.rate() {
            specs.push("aresample=resampler=soxr".into());
        }

        tracing::debug!(?specs);
        let spec = if specs.is_empty() { "anull".into() } else { specs.join(",").into() };

        Self { graph: avfilter::Graph::new(), spec }
    }
}

impl Filter {
    pub fn new(graph: &mut Graph, input: &Input, output: &Output) -> Result<Self, Error> {
        let decoder = &input.decoder;
        let encoder = &output.encoder;

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
        if encoder.frame_size() > 0 {
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
                    unsafe {
                        (*frame.as_mut_ptr()).time_base = sink.time_base().into();
                    }
                    output.encode(Some(&mut frame))?;
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
        bit_rate: u32,
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

            let mut transcoder = Self::new(&path.input, sink, bit_rate, offset)?;
            transcoder.transcode()?;
            atomic_cache.map(AtomicWriteFile::commit).transpose()?;
            Ok(())
        });

        (rx, handle)
    }

    fn new(input: &str, sink: Sink, bit_rate: u32, offset: u32) -> Result<Self, Error> {
        let input = Input::new(input)?;
        let output = Output::new(sink, bit_rate, &input.decoder)?;
        let graph = Graph::new(&input, &output, offset);
        Ok(Self { input, output, graph })
    }

    #[cfg_attr(
        not(coverage_nightly),
        instrument(skip_all, ret(level = "debug"), err(Debug, level = "debug"))
    )]
    pub fn transcode(&mut self) -> Result<(), Error> {
        let mut filter = Filter::new(&mut self.graph, &self.input, &self.output)?;

        let mut packet = AvPacket::empty();
        loop {
            match packet.read(&mut self.input.context) {
                Err(AvError::Eof) => break self.input.decoder.send_eof()?,
                Err(error) => return Err(error.into()),
                Ok(()) => {
                    // Ignore non audio stream packets.
                    if packet.stream() != self.input.index {
                        continue;
                    }
                    self.input.decoder.send_packet(&packet)?;
                }
            }

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
                        let frame_timestamp = frame.timestamp();
                        frame.set_pts(frame_timestamp);
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
            bit_rate: u32,
            offset: u32,
        ) -> Vec<u8> {
            let (rx, handle) = Transcoder::spawn(
                config,
                Path { input: input.into(), cache: None },
                format,
                bit_rate,
                offset,
            );
            let data = rx.into_stream().map(stream::iter).flatten().collect().await;
            handle.await.unwrap().unwrap();
            data
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use nghe_api::common::format;
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::assets;
    use crate::{config, init_ffmpeg};

    #[rstest]
    #[case(format::Transcode::Aac, 128)]
    #[case(format::Transcode::Mp3, 320)]
    #[cfg_attr(debug_assertions, case(format::Transcode::Opus, 64))]
    #[case(format::Transcode::Wav, 0)]
    #[case(format::Transcode::Wma, 128)]
    #[tokio::test]
    async fn test_transcode(
        #[case] format: format::Transcode,
        #[case] bit_rate: u32,
        #[values(0, 5)] offset: u32,
    ) {
        let config = config::Transcode {
            log_level: ffmpeg_next::log::Level::Trace.into(),
            ..Default::default()
        };
        init_ffmpeg(&config).unwrap();

        let input = assets::path(audio::Format::Flac);
        let data = Transcoder::spawn_collect(&config, input, format, bit_rate, offset).await;

        let transcoded = assets::transcoded(format, offset);
        if tokio::fs::try_exists(&transcoded).await.unwrap() {
            let transcoded = tokio::fs::read(transcoded).await.unwrap();
            assert!(data == transcoded, "Transcoded file does not match expected file");
        } else {
            tokio::fs::create_dir_all(transcoded.parent().unwrap()).await.unwrap();
            tokio::fs::write(transcoded, data).await.unwrap();
        }
    }

    #[cfg(hearing_test)]
    #[rstest]
    #[case(format::Transcode::Mp3, 320)]
    #[case(format::Transcode::Opus, 64)]
    #[tokio::test]
    async fn test_hearing(
        #[case] format: format::Transcode,
        #[case] bit_rate: u32,
        #[values(0, 10)] offset: u32,
    ) {
        let config = config::Transcode {
            log_level: ffmpeg_next::log::Level::Trace.into(),
            ..Default::default()
        };
        init_ffmpeg(&config).unwrap();

        let input = env!("NGHE_HEARING_TEST_INPUT");
        let data = Transcoder::spawn_collect(&config, input, format, bit_rate, offset).await;

        tokio::fs::write(
            typed_path::Utf8PlatformPath::new(env!("NGHE_HEARING_TEST_OUTPUT"))
                .join(concat_string!(bit_rate.to_string(), "-", offset.to_string()))
                .with_extension(format.as_ref()),
            &data,
        )
        .await
        .unwrap();
    }
}
