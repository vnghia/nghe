use anyhow::{Context, Result};
use concat_string::concat_string;
use crossfire::{channel::MPSCShared, mpsc};
use rsmpeg::{
    avcodec::{AVCodec, AVCodecContext},
    avfilter::{AVFilter, AVFilterContextMut, AVFilterGraph, AVFilterInOut},
    avformat::{
        AVFormatContextInput, AVFormatContextOutput, AVIOContextContainer, AVIOContextCustom,
    },
    avutil::{get_sample_fmt_name, ra, AVFrame, AVMem},
    error::RsmpegError,
    ffi,
};
use std::{
    ffi::{CStr, CString},
    sync::atomic::{AtomicI64, Ordering},
};

use crate::OSError;

fn open_input_file(path: &CStr) -> Result<(AVFormatContextInput, AVCodecContext, usize)> {
    let input_fmt_ctx =
        AVFormatContextInput::open(path, None, &mut None).context("could not open input file")?;

    let (audio_idx, dec_codec) = input_fmt_ctx
        .find_best_stream(ffi::AVMediaType_AVMEDIA_TYPE_AUDIO)?
        .context("could not file audio index")?;
    let stream = &input_fmt_ctx.streams()[audio_idx];

    let mut dec_ctx = AVCodecContext::new(&dec_codec);
    dec_ctx
        .apply_codecpar(&stream.codecpar())
        .context("could not apply codecpar to decoding context")?;
    dec_ctx.open(None).context("could not open input codec")?;
    dec_ctx.set_pkt_timebase(stream.time_base);
    dec_ctx.set_bit_rate(input_fmt_ctx.bit_rate);

    Ok((input_fmt_ctx, dec_ctx, audio_idx))
}

fn make_output_io_context<S: MPSCShared + 'static>(
    buffer_size: usize,
    tx: mpsc::TxBlocking<Vec<u8>, S>,
) -> AVIOContextContainer {
    AVIOContextContainer::Custom(AVIOContextCustom::alloc_context(
        AVMem::new(buffer_size),
        true,
        vec![],
        None,
        Some(Box::new(move |_, data| match tx.send(data.to_vec()) {
            Ok(()) => data.len() as i32,
            Err(_) => ffi::AVERROR_EXTERNAL,
        })),
        None,
    ))
}

fn open_output_file(
    path: &CStr,
    dec_ctx: &AVCodecContext,
    output_bitrate: u32,
    io_ctx: Option<AVIOContextContainer>,
) -> Result<(AVFormatContextOutput, AVCodecContext)> {
    let mut output_fmt_ctx =
        AVFormatContextOutput::create(path, io_ctx).context("could not open output file")?;

    let enc_codec = AVCodec::find_encoder(output_fmt_ctx.oformat().audio_codec)
        .context("could not find output codec")?;
    let output_sample_rate = if enc_codec.id == ffi::AVCodecID_AV_CODEC_ID_OPUS {
        48000 // libopus recommended sample rate
    } else {
        dec_ctx.sample_rate
    };

    let mut enc_ctx = AVCodecContext::new(&enc_codec);
    enc_ctx.set_ch_layout(dec_ctx.ch_layout);
    enc_ctx.set_sample_fmt(
        enc_codec
            .sample_fmts()
            .ok_or_else(|| OSError::NotFound("could not get sample formats".into()))?[0],
    );
    enc_ctx.set_sample_rate(output_sample_rate);
    enc_ctx.set_bit_rate(output_bitrate as i64);
    enc_ctx.set_time_base(ra(1, output_sample_rate));
    // Some container formats (like MP4) require global headers to be present.
    // Mark the encoder so that it behaves accordingly.
    if output_fmt_ctx.oformat().flags & ffi::AVFMT_GLOBALHEADER as i32 != 0 {
        enc_ctx.set_flags(enc_ctx.flags | ffi::AV_CODEC_FLAG_GLOBAL_HEADER as i32);
    }
    // Open the encoder for the audio stream to use it later.
    enc_ctx.open(None)?;

    {
        // Create a new audio stream in the output file container.
        let mut stream = output_fmt_ctx.new_stream();
        stream.set_codecpar(enc_ctx.extract_codecpar());
        // Set the sample rate for the container.
        stream.set_time_base(enc_ctx.time_base);
    }

    // Write the header of the output file container.
    output_fmt_ctx
        .write_header(&mut None)
        .context("could not write output file header")?;

    Ok((output_fmt_ctx, enc_ctx))
}

fn init_filter<'a>(
    filter_graph: &'a mut AVFilterGraph,
    dec_ctx: &mut AVCodecContext,
    enc_ctx: &mut AVCodecContext,
    filter_spec: &CStr,
) -> Result<(AVFilterContextMut<'a>, AVFilterContextMut<'a>)> {
    let src = AVFilter::get_by_name(c"abuffer").unwrap();
    let sink = AVFilter::get_by_name(c"abuffersink").unwrap();

    let filter_args = concat_string!(
        "time_base=",
        dec_ctx.pkt_timebase.num.to_string(),
        "/",
        dec_ctx.pkt_timebase.den.to_string(),
        ":sample_rate=",
        dec_ctx.sample_rate.to_string(),
        ":sample_fmt=",
        // We can unwrap here, because we are sure that the given
        // sample_fmt is valid.
        get_sample_fmt_name(dec_ctx.sample_fmt)
            .unwrap()
            .to_string_lossy(),
        ":channel_layout=",
        dec_ctx.ch_layout().describe().unwrap().to_string_lossy()
    );
    let filter_cargs = &CString::new(filter_args).unwrap();

    let mut src_ctx = filter_graph
        .create_filter_context(&src, c"in", Some(filter_cargs))
        .context("could not create audio buffer source")?;

    let mut sink_ctx = filter_graph
        .create_filter_context(&sink, c"out", None)
        .context("could create audio buffer sink")?;
    sink_ctx
        .opt_set_bin(c"sample_fmts", &enc_ctx.sample_fmt)
        .context("could not set output sample format")?;
    sink_ctx
        .opt_set(c"ch_layouts", &enc_ctx.ch_layout().describe().unwrap())
        .context("could not set output channel layout")?;
    sink_ctx
        .opt_set_bin(c"sample_rates", &enc_ctx.sample_rate)
        .context("could not set output sample rate")?;

    // Yes. The output name is in.
    let outputs = AVFilterInOut::new(c"in", &mut src_ctx, 0);
    let inputs = AVFilterInOut::new(c"out", &mut sink_ctx, 0);
    let (_inputs, _outputs) = filter_graph.parse_ptr(filter_spec, Some(inputs), Some(outputs))?;

    filter_graph.config()?;

    Ok((src_ctx, sink_ctx))
}

fn encode_audio_frame(
    mut frame: Option<AVFrame>,
    enc_ctx: &mut AVCodecContext,
    output_fmt_ctx: &mut AVFormatContextOutput,
) -> Result<()> {
    static PTS: AtomicI64 = AtomicI64::new(0);

    if let Some(frame) = frame.as_mut() {
        frame.set_pts(PTS.fetch_add(frame.nb_samples as i64, Ordering::Relaxed));
    }

    // Check for errors, but proceed with fetching encoded samples if the
    // encoder signals that it has nothing more to encode.
    match enc_ctx.send_frame(frame.as_ref()) {
        Err(err) if err.raw_error().is_some_and(|err| err == ffi::AVERROR_EOF) => (),
        r => r?,
    };

    loop {
        let mut packet = match enc_ctx.receive_packet() {
            Ok(packet) => packet,
            Err(RsmpegError::EncoderDrainError) | Err(RsmpegError::EncoderFlushedError) => {
                break Ok(());
            }
            Err(e) => anyhow::bail!(e),
        };

        packet.set_stream_index(0);
        packet.rescale_ts(enc_ctx.time_base, output_fmt_ctx.streams()[0].time_base);

        output_fmt_ctx
            .interleaved_write_frame(&mut packet)
            .context("could not write frame")?;
    }
}

fn filter_and_encode_audio_frame(
    frame: Option<AVFrame>,
    src_ctx: &mut AVFilterContextMut,
    sink_ctx: &mut AVFilterContextMut,
    enc_ctx: &mut AVCodecContext,
    output_fmt_ctx: &mut AVFormatContextOutput,
) -> Result<()> {
    src_ctx
        .buffersrc_add_frame(frame, None)
        .context("could not submit frame to the filter graph")?;

    loop {
        let frame = match sink_ctx.buffersink_get_frame(None) {
            Ok(frame) => frame,
            Err(RsmpegError::BufferSinkDrainError) | Err(RsmpegError::BufferSinkEofError) => {
                break Ok(())
            }
            Err(err) => anyhow::bail!(err),
        };
        encode_audio_frame(Some(frame), enc_ctx, output_fmt_ctx)?;
    }
}

fn flush_encoder(
    enc_ctx: &mut AVCodecContext,
    output_fmt_ctx: &mut AVFormatContextOutput,
) -> Result<()> {
    if enc_ctx.codec().capabilities & ffi::AV_CODEC_CAP_DELAY as i32 != 0 {
        encode_audio_frame(None, enc_ctx, output_fmt_ctx)
    } else {
        Ok(())
    }
}

fn transcode_with_io_context(
    input_path: &CStr,
    output_path: &CStr,
    output_bit_rate: u32,
    output_time_offset: Option<u32>,
    io_ctx: Option<AVIOContextContainer>,
) -> Result<()> {
    let (mut input_fmt_ctx, mut dec_ctx, audio_idx) = open_input_file(input_path)?;
    let (mut output_fmt_ctx, mut enc_ctx) =
        open_output_file(output_path, &dec_ctx, output_bit_rate, io_ctx)?;

    let mut filter_specs = vec![];
    if let Some(output_time_offset) = output_time_offset {
        filter_specs.push(concat_string!(
            "atrim=start=",
            output_time_offset.to_string()
        ));
    }
    filter_specs.push("aresample=resampler=soxr".to_owned());
    if enc_ctx.frame_size > 0 {
        filter_specs.push(concat_string!(
            "asetnsamples=n=",
            enc_ctx.frame_size.to_string(),
            ":p=0"
        ))
    }
    let filter_spec = filter_specs.join(",");

    let mut filter_graph = AVFilterGraph::new();
    let (mut src_ctx, mut sink_ctx) = init_filter(
        &mut filter_graph,
        &mut dec_ctx,
        &mut enc_ctx,
        &CString::new(filter_spec).unwrap(),
    )?;

    loop {
        let packet = match input_fmt_ctx.read_packet() {
            Err(err) if err.raw_error().is_some_and(|err| err == ffi::AVERROR_EOF) => None,
            r => r.context("could not read input frame")?,
        };

        // Ignore non audio stream packets.
        if packet
            .as_ref()
            .is_some_and(|p| p.stream_index as usize != audio_idx)
        {
            continue;
        }

        dec_ctx
            .send_packet(packet.as_ref())
            .context("could not send packet for decoding")?;

        // If packet is none, it means that we are at EOF.
        // The decoder is flush as above.
        if packet.is_none() {
            break;
        }

        loop {
            let input_frame = match dec_ctx.receive_frame() {
                Ok(frame) => frame,
                // There is nothing to read anymore.
                Err(RsmpegError::DecoderDrainError) | Err(RsmpegError::DecoderFlushedError) => {
                    break;
                }
                Err(e) => anyhow::bail!(e),
            };

            filter_and_encode_audio_frame(
                Some(input_frame),
                &mut src_ctx,
                &mut sink_ctx,
                &mut enc_ctx,
                &mut output_fmt_ctx,
            )?;
        }
    }

    // Flush the filter graph by pushing EOF packet to buffer_src_context.
    // Flush the encoder by pushing EOF frame to encode_context.
    filter_and_encode_audio_frame(
        None,
        &mut src_ctx,
        &mut sink_ctx,
        &mut enc_ctx,
        &mut output_fmt_ctx,
    )
    .context("can not flush the filter")?;
    flush_encoder(&mut enc_ctx, &mut output_fmt_ctx).context("can not flush the encoder")?;

    Ok(())
}

pub fn transcode<S: MPSCShared + 'static>(
    input_path: &CStr,
    output_path: &CStr,
    output_bit_rate: u32,
    output_time_offset: Option<u32>,
    tx: mpsc::TxBlocking<Vec<u8>, S>,
) {
    if let Err(err) = transcode_with_io_context(
        input_path,
        output_path,
        output_bit_rate,
        output_time_offset,
        Some(make_output_io_context(32 * 1024, tx)),
    ) {
        tracing::error!("error while transcoding: {:?}", err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        song::file_type::SONG_FILE_TYPES,
        test::{asset::get_media_asset_path, TemporaryFs},
    };

    use fake::{Fake, Faker};
    use lofty::FileType;
    use std::path::Path;

    const OUTPUT_EXTENSIONS: &[&str] = &["mp3", "aac", "opus"];
    const OUTPUT_BITRATE: &[u32] = &[32000, 64000, 128000, 192000, 320000];
    const OUTPUT_TIME_OFFSETS: &[Option<u32>] = &[None, Some(5), Some(u32::MAX)];

    fn path_to_cstring<P: AsRef<Path>>(path: P) -> CString {
        CString::new(path.as_ref().to_str().unwrap()).unwrap()
    }

    fn get_media_asset_cstring(file_type: &FileType) -> CString {
        path_to_cstring(get_media_asset_path(file_type))
    }

    async fn wrap_transcode(
        input_path: &CStr,
        output_path: &CStr,
        output_bit_rate: u32,
        output_time_offset: Option<u32>,
    ) -> Vec<u8> {
        let (tx, rx) = mpsc::bounded_tx_blocking_rx_future(1);
        let input_path = input_path.to_owned();
        let output_path = output_path.to_owned();

        let transcode_thread = tokio::task::spawn_blocking(move || {
            transcode_with_io_context(
                &input_path,
                &output_path,
                output_bit_rate,
                output_time_offset,
                Some(make_output_io_context(32 * 1024, tx)),
            )
        });

        let mut result = vec![];
        while let Ok(r) = rx.recv().await {
            result.extend_from_slice(&r);
        }

        transcode_thread.await.unwrap().unwrap();
        result
    }

    #[tokio::test]
    async fn test_transcode() {
        let fs = TemporaryFs::new();

        for file_type in SONG_FILE_TYPES {
            let media_path = get_media_asset_cstring(&file_type);
            for output_extension in OUTPUT_EXTENSIONS {
                for output_bitrate in OUTPUT_BITRATE {
                    for output_time_offset in OUTPUT_TIME_OFFSETS {
                        let output_path = path_to_cstring(
                            fs.root_path()
                                .join(Faker.fake::<String>())
                                .with_extension(output_extension),
                        );
                        wrap_transcode(
                            &media_path,
                            &output_path,
                            *output_bitrate,
                            *output_time_offset,
                        )
                        .await;
                    }
                }
            }
        }
    }
}
