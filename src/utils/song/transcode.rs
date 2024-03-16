use anyhow::{Context, Result};
use rsmpeg::{
    avcodec::{AVCodec, AVCodecContext},
    avformat::{AVFormatContextInput, AVFormatContextOutput},
    avutil::{ra, AVAudioFifo, AVFrame, AVSamples},
    error::RsmpegError,
    ffi,
    swresample::SwrContext,
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

fn open_output_file(
    path: &CStr,
    dec_ctx: &AVCodecContext,
    output_bitrate: i64,
) -> Result<(AVFormatContextOutput, AVCodecContext)> {
    let mut output_fmt_ctx =
        AVFormatContextOutput::create(path, None).context("could not open output file")?;

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
    enc_ctx.set_bit_rate(output_bitrate);
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

    Ok((output_fmt_ctx, enc_ctx))
}

fn init_resampler(dec_ctx: &AVCodecContext, enc_ctx: &AVCodecContext) -> Result<SwrContext> {
    let mut resample_context = SwrContext::new(
        &enc_ctx.ch_layout,
        enc_ctx.sample_fmt,
        enc_ctx.sample_rate,
        &dec_ctx.ch_layout,
        dec_ctx.sample_fmt,
        dec_ctx.sample_rate,
    )
    .context("could not allocate resample context")?;

    unsafe {
        // TODO: use c".." in Rust 1.77
        let resampler_key = CString::new("resampler").unwrap();
        let ret = ffi::av_opt_set_int(
            resample_context.as_mut_ptr() as *mut _,
            resampler_key.as_ptr() as *const _,
            ffi::SwrEngine_SWR_ENGINE_SOXR as i64,
            0,
        );
        if ret != 0 {
            anyhow::bail!(OSError::InvalidParameter(
                "can not set sampler to soxr".into()
            ))
        }
    }

    resample_context
        .init()
        .context("could not open resample context")?;

    Ok(resample_context)
}

fn add_samples_to_fifo(
    fifo: &mut AVAudioFifo,
    converted_input_samples: &AVSamples,
    frame_size: i32,
) -> Result<()> {
    fifo.realloc(fifo.size() + frame_size);
    unsafe { fifo.write(converted_input_samples.audio_data.as_ptr(), frame_size) }
        .context("could not write data to FIFO")?;
    Ok(())
}

fn init_output_frame(
    frame_size: i32,
    ch_layout: ffi::AVChannelLayout,
    enc_ctx: &AVCodecContext,
) -> Result<AVFrame> {
    let mut frame = AVFrame::new();

    frame.set_nb_samples(frame_size);
    frame.set_ch_layout(ch_layout);
    frame.set_format(enc_ctx.sample_fmt);
    frame.set_sample_rate(enc_ctx.sample_rate);

    frame
        .get_buffer(0)
        .context("could not allocate output frame samples")?;

    Ok(frame)
}

fn encode_audio_frame(
    mut frame: Option<AVFrame>,
    output_fmt_ctx: &mut AVFormatContextOutput,
    enc_ctx: &mut AVCodecContext,
) -> Result<bool> {
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

    match enc_ctx.receive_packet() {
        Ok(mut packet) => output_fmt_ctx
            .write_frame(&mut packet)
            .context("could not write frame")
            .map(|()| true),
        Err(RsmpegError::EncoderDrainError) | Err(RsmpegError::EncoderFlushedError) => Ok(false),
        Err(err) => anyhow::bail!(err),
    }
}

fn load_encode_and_write(
    fifo: &mut AVAudioFifo,
    output_fmt_ctx: &mut AVFormatContextOutput,
    enc_ctx: &mut AVCodecContext,
) -> Result<()> {
    let frame_size = fifo.size().min(enc_ctx.frame_size);

    let mut frame = init_output_frame(
        frame_size,
        enc_ctx.ch_layout().clone().into_inner(),
        enc_ctx,
    )?;

    if unsafe { fifo.read(frame.data_mut().as_mut_ptr(), frame_size)? } < frame_size {
        anyhow::bail!(OSError::InvalidParameter(
            "could not read data from FIFO".into()
        ));
    }
    encode_audio_frame(Some(frame), output_fmt_ctx, enc_ctx)?;

    Ok(())
}

pub fn transcode(input_path: &CStr, output_path: &CStr, output_bit_rate: i64) -> Result<()> {
    let (mut input_fmt_ctx, mut dec_ctx, audio_idx) = open_input_file(input_path)?;
    let (mut output_fmt_ctx, mut enc_ctx) =
        open_output_file(output_path, &dec_ctx, output_bit_rate)?;
    let mut resample_context = init_resampler(&dec_ctx, &enc_ctx)?;

    // Initialize the FIFO buffer to store audio samples to be encoded.
    let mut fifo = AVAudioFifo::new(enc_ctx.sample_fmt, enc_ctx.ch_layout.nb_channels, 1);

    // Write the header of the output file container.
    output_fmt_ctx
        .write_header(&mut None)
        .context("could not write output file header")?;

    // Loop as long as we have input samples to read or output samples to write.
    // Abort as soon as we have neither.
    loop {
        let output_frame_size = enc_ctx.frame_size;

        let finished = loop {
            // We have enough data to encode
            if fifo.size() >= output_frame_size {
                break false;
            }

            // read_decode_convert_and_store

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
            // It is the same as setting `finished` in the original example.
            // The decoder is flush as above.
            if packet.is_none() {
                break true;
            }

            let input_frame = match dec_ctx.receive_frame() {
                // `data_present` set to 1.
                Ok(frame) => frame,
                // There is nothing to read anymore.
                // Breaking here is the same as setting `data_present` to 0 in the original example.
                Err(RsmpegError::DecoderDrainError) => {
                    break false;
                }
                // Reaching EOF. Stop decoding
                Err(RsmpegError::DecoderFlushedError) => {
                    break true;
                }
                Err(e) => anyhow::bail!(e),
            };

            let mut output_samples = AVSamples::new(
                enc_ctx.ch_layout.nb_channels,
                input_frame.nb_samples,
                enc_ctx.sample_fmt,
                0,
            )
            .context("could not create samples buffer")?;

            unsafe {
                resample_context
                    .convert(
                        output_samples.audio_data.as_mut_ptr(),
                        output_samples.nb_samples,
                        input_frame.extended_data as *const _,
                        input_frame.nb_samples,
                    )
                    .context("could not convert input samples")?;
            }

            add_samples_to_fifo(&mut fifo, &output_samples, input_frame.nb_samples)?;
        };

        // Write frame as much as possible.
        while (fifo.size() >= output_frame_size) || (finished && fifo.size() > 0) {
            load_encode_and_write(&mut fifo, &mut output_fmt_ctx, &mut enc_ctx)?;
        }

        if finished {
            // Flush the encoder as it may have delayed frames.
            loop {
                if !encode_audio_frame(None, &mut output_fmt_ctx, &mut enc_ctx)? {
                    break;
                }
            }
            break;
        }
    }

    output_fmt_ctx.write_trailer()?;

    Ok(())
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
    use std::{ffi::CString, path::Path};

    const OUTPUT_EXTENSIONS: &[&str] = &["mp3", "aac", "opus"];
    const OUTPUT_BITRATE: &[i64] = &[32000, 64000, 96000, 128000, 160000, 192000, 256000, 320000];

    fn path_to_cstring<P: AsRef<Path>>(path: P) -> CString {
        CString::new(path.as_ref().to_str().unwrap()).unwrap()
    }

    fn get_media_asset_cstring(file_type: &FileType) -> CString {
        path_to_cstring(get_media_asset_path(file_type))
    }

    #[test]
    fn test_transcode() {
        let fs = TemporaryFs::new();

        for file_type in SONG_FILE_TYPES {
            let media_path = get_media_asset_cstring(&file_type);
            for output_extension in OUTPUT_EXTENSIONS {
                let output_path = path_to_cstring(
                    fs.root_path()
                        .join(Faker.fake::<String>())
                        .with_extension(output_extension),
                );
                for output_bitrate in OUTPUT_BITRATE {
                    transcode(&media_path, &output_path, *output_bitrate).unwrap();
                }
            }
        }
    }
}
