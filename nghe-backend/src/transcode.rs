use std::borrow::Cow;
use std::ffi::{CStr, CString};

use concat_string::concat_string;
use rsmpeg::avcodec::{AVCodec, AVCodecContext};
use rsmpeg::avfilter::{AVFilter, AVFilterContextMut, AVFilterGraph, AVFilterInOut};
use rsmpeg::avformat::{AVFormatContextInput, AVFormatContextOutput};
use rsmpeg::avutil::AVFrame;
use rsmpeg::error::RsmpegError;
use rsmpeg::{avutil, ffi, UnsafeDerefMut};

use crate::Error;

struct Input {
    context: AVFormatContextInput,
    decoder: AVCodecContext,
    index: i32,
}

struct Output {
    context: AVFormatContextOutput,
    encoder: AVCodecContext,
}

struct Graph {
    filter: AVFilterGraph,
    spec: Cow<'static, CStr>,
}

struct Filter<'a> {
    source: AVFilterContextMut<'a>,
    sink: AVFilterContextMut<'a>,
}

pub struct Transcoder {
    input: Input,
    output: Output,
    graph: Graph,
}

impl Input {
    fn new(input: &CStr) -> Result<Self, Error> {
        let context = AVFormatContextInput::open(input, None, &mut None)?;
        let (index, codec) = context
            .find_best_stream(ffi::AVMEDIA_TYPE_AUDIO)?
            .ok_or_else(|| Error::MediaAudioTrackMissing)?;
        let stream = &context.streams()[index];

        let mut decoder = AVCodecContext::new(&codec);
        decoder.apply_codecpar(&stream.codecpar())?;
        decoder.open(None)?;
        decoder.set_pkt_timebase(stream.time_base);
        decoder.set_bit_rate(context.bit_rate);

        Ok(Self { context, decoder, index: index.try_into()? })
    }
}

impl Output {
    fn new(output: &CStr, bitrate: u32, decoder: &AVCodecContext) -> Result<Self, Error> {
        let mut context = AVFormatContextOutput::create(output, None)?;

        if cfg!(test) {
            // Set bitexact for deterministic transcoding output.
            unsafe {
                context.deref_mut().flags |= ffi::AVFMT_FLAG_BITEXACT as i32;
            }
        }

        let codec = AVCodec::find_encoder(context.oformat().audio_codec)
            .ok_or_else(|| Error::TranscodeOutputFormatNotSupported)?;

        // bit to kbit
        let bitrate = bitrate * 1000;
        // Opus sample rate will always be 48000Hz.
        let sample_rate =
            if codec.id == ffi::AV_CODEC_ID_OPUS { 48000 } else { decoder.sample_rate };

        let mut encoder = AVCodecContext::new(&codec);
        encoder.set_ch_layout(decoder.ch_layout);
        encoder.set_sample_fmt(
            codec.sample_fmts().ok_or_else(|| Error::TranscodeEncoderSampleFmtsMissing)?[0],
        );
        encoder.set_sample_rate(sample_rate);
        encoder.set_bit_rate(bitrate.into());
        encoder.set_time_base(avutil::ra(1, sample_rate));

        // Some formats want stream headers to be separate.
        if context.oformat().flags & ffi::AVFMT_GLOBALHEADER as i32 != 0 {
            encoder.set_flags(encoder.flags | ffi::AV_CODEC_FLAG_GLOBAL_HEADER as i32);
        }

        encoder.open(None)?;
        {
            let mut stream = context.new_stream();
            stream.set_codecpar(encoder.extract_codecpar());
            stream.set_time_base(encoder.time_base);
        }
        context.write_header(&mut None)?;

        Ok(Self { context, encoder })
    }

    fn encode(&mut self, frame: Option<&AVFrame>) -> Result<(), Error> {
        self.encoder.send_frame(frame)?;

        loop {
            let mut packet = match self.encoder.receive_packet() {
                Err(RsmpegError::EncoderDrainError | RsmpegError::EncoderFlushedError) => {
                    return Ok(());
                }
                result => result?,
            };

            packet.set_stream_index(0);
            packet.rescale_ts(self.encoder.time_base, self.context.streams()[0].time_base);
            self.context.interleaved_write_frame(&mut packet)?;
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        if self.encoder.codec().capabilities & ffi::AV_CODEC_CAP_DELAY as i32 != 0 {
            self.encode(None)
        } else {
            Ok(())
        }
    }
}

impl Graph {
    fn new(decoder: &AVCodecContext, encoder: &AVCodecContext, offset: u32) -> Result<Self, Error> {
        let mut specs: Vec<Cow<'static, str>> = vec![];
        if offset > 0 {
            specs.push(concat_string!("atrim=start=", offset.to_string()).into());
        }
        if decoder.sample_rate != encoder.sample_rate {
            specs.push("aresample=resampler=soxr".into());
        }
        if encoder.frame_size > 0 {
            specs.push(
                concat_string!("asetnsamples=n=", encoder.frame_size.to_string(), ":p=0").into(),
            );
        }

        let spec =
            if specs.is_empty() { c"anull".into() } else { CString::new(specs.join(","))?.into() };

        Ok(Self { filter: AVFilterGraph::new(), spec })
    }
}

impl<'graph> Filter<'graph> {
    pub fn new(
        graph: &'graph Graph,
        decoder: &AVCodecContext,
        encoder: &AVCodecContext,
    ) -> Result<Self, Error> {
        let source_ref = AVFilter::get_by_name(c"abuffer")
            .ok_or_else(|| Error::TranscodeAVFilterMissing("abuffer"))?;
        let sink_ref = AVFilter::get_by_name(c"abuffersink")
            .ok_or_else(|| Error::TranscodeAVFilterMissing("abuffersink"))?;

        let source_arg = concat_string!(
            "time_base=",
            decoder.pkt_timebase.num.to_string(),
            "/",
            decoder.pkt_timebase.den.to_string(),
            ":sample_rate=",
            decoder.sample_rate.to_string(),
            ":sample_fmt=",
            avutil::get_sample_fmt_name(decoder.sample_fmt)
                .ok_or_else(|| Error::TranscodeSampleFmtNameMissing(decoder.sample_fmt))?
                .to_str()?,
            ":channel_layout=",
            decoder.ch_layout().describe()?.to_str()?
        );
        let source_arg = CString::new(source_arg)?;
        let mut source =
            graph.filter.create_filter_context(&source_ref, c"in", Some(&source_arg))?;

        let mut sink = graph.filter.create_filter_context(&sink_ref, c"out", None)?;
        sink.opt_set_bin(c"sample_rates", &encoder.sample_rate)?;
        sink.opt_set_bin(c"sample_fmts", &encoder.sample_fmt)?;
        sink.opt_set(c"ch_layouts", &encoder.ch_layout().describe()?)?;

        // Yes. The output name is in.
        let outputs = AVFilterInOut::new(c"in", &mut source, 0);
        let inputs = AVFilterInOut::new(c"out", &mut sink, 0);
        graph.filter.parse_ptr(&graph.spec, Some(inputs), Some(outputs))?;

        graph.filter.config()?;

        Ok(Self { source, sink })
    }

    fn filter_and_encode(
        &mut self,
        output: &mut Output,
        frame: Option<AVFrame>,
    ) -> Result<(), Error> {
        self.source.buffersrc_add_frame(frame, None)?;

        loop {
            let frame = match self.sink.buffersink_get_frame(None) {
                Err(RsmpegError::BufferSinkDrainError | RsmpegError::BufferSinkEofError) => {
                    break Ok(());
                }
                result => result?,
            };
            output.encode(Some(&frame))?;
        }
    }
}

impl Transcoder {
    pub fn new(input: &CStr, output: &CStr, bitrate: u32, offset: u32) -> Result<Self, Error> {
        let input = Input::new(input)?;
        let output = Output::new(output, bitrate, &input.decoder)?;
        let graph = Graph::new(&input.decoder, &output.encoder, offset)?;
        Ok(Self { input, output, graph })
    }

    pub fn transcode(&mut self) -> Result<(), Error> {
        let mut filter = Filter::new(&self.graph, &self.input.decoder, &self.output.encoder)?;

        loop {
            let packet = self.input.context.read_packet()?;

            // Ignore non audio stream packets.
            if packet.as_ref().is_some_and(|p| p.stream_index != self.input.index) {
                continue;
            }

            self.input.decoder.send_packet(packet.as_ref())?;

            // If packet is none, we are at input EOF.
            // The decoder is flushed by passing a none packet as above.
            if packet.is_none() {
                break;
            }

            loop {
                let frame = match self.input.decoder.receive_frame() {
                    Err(RsmpegError::DecoderDrainError | RsmpegError::DecoderFlushedError) => {
                        break;
                    }
                    result => result?,
                };
                filter.filter_and_encode(&mut self.output, Some(frame))?;
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
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::filesystem;

    #[cfg(hearing_test)]
    #[rstest]
    #[case("opus", 64)]
    #[case("mp3", 320)]
    fn test_hearing(#[case] format: &str, #[case] bitrate: u32, #[values(0, 10)] offset: u32) {
        let input = CString::new(env!("NGHE_HEARING_TEST_INPUT")).unwrap();
        let output = CString::new(
            filesystem::path::Local::from_str(env!("NGHE_HEARING_TEST_OUTPUT"))
                .join(concat_string!(bitrate.to_string(), "-", offset.to_string()))
                .with_extension(format)
                .into_string(),
        )
        .unwrap();
        Transcoder::new(&input, &output, bitrate, offset).unwrap().transcode().unwrap();
    }
}
