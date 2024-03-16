// https://github.com/larksuite/rsmpeg/blob/master/tests/transcode_aac.rs

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
    let fmt_ctx_in =
        AVFormatContextInput::open(path, None, &mut None).context("could not open input file")?;

    let (audio_idx, dec_codec) = fmt_ctx_in
        .find_best_stream(ffi::AVMediaType_AVMEDIA_TYPE_AUDIO)?
        .context("could not file audio index")?;

    let stream = &fmt_ctx_in.streams()[audio_idx];
    let mut dec_ctx = AVCodecContext::new(&dec_codec);
    dec_ctx.apply_codecpar(&stream.codecpar())?;
    dec_ctx.open(None).context("could not open input codec")?;
    dec_ctx.set_pkt_timebase(stream.time_base);
    dec_ctx.set_bit_rate(fmt_ctx_in.bit_rate);

    Ok((fmt_ctx_in, dec_ctx, audio_idx))
}

fn open_output_file(
    path: &CStr,
    dec_ctx: &AVCodecContext,
    output_bit_rate: i64,
) -> Result<(AVFormatContextOutput, AVCodecContext)> {
    let mut fmt_ctx_out =
        AVFormatContextOutput::create(path, None).context("could not open output file")?;

    let enc_codec = AVCodec::find_encoder(fmt_ctx_out.oformat().audio_codec)
        .context("could not find output codec")?;
    let mut enc_ctx = AVCodecContext::new(&enc_codec);

    enc_ctx.set_ch_layout(dec_ctx.ch_layout);
    enc_ctx.set_sample_fmt(
        enc_codec
            .sample_fmts()
            .ok_or_else(|| OSError::NotFound("could not get sample formats".into()))?[0],
    );

    let output_sample_rate = if enc_codec.id == ffi::AVCodecID_AV_CODEC_ID_OPUS {
        // libopus recommended sample rate
        48000
    } else {
        dec_ctx.sample_rate
    };

    enc_ctx.set_sample_rate(output_sample_rate);
    enc_ctx.set_bit_rate(output_bit_rate);

    // Open the encoder for the audio stream to use it later.
    enc_ctx.open(None)?;

    {
        // Create a new audio stream in the output file container.
        let mut stream = fmt_ctx_out.new_stream();
        stream.set_codecpar(enc_ctx.extract_codecpar());
        // Set the sample rate for the container.
        stream.set_time_base(ra(1, output_sample_rate));
    }

    Ok((fmt_ctx_out, enc_ctx))
}

fn init_resampler(
    dec_ctx: &mut AVCodecContext,
    enc_ctx: &mut AVCodecContext,
) -> Result<SwrContext> {
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
    samples_buffer: &AVSamples,
    frame_size: i32,
) -> Result<()> {
    fifo.realloc(fifo.size() + frame_size);
    unsafe { fifo.write(samples_buffer.audio_data.as_ptr(), frame_size) }
        .context("could not write data to FIFO")?;
    Ok(())
}

fn init_output_frame(
    nb_samples: i32,
    ch_layout: ffi::AVChannelLayout,
    sample_fmt: i32,
    sample_rate: i32,
) -> Result<AVFrame> {
    let mut frame = AVFrame::new();

    frame.set_nb_samples(nb_samples);
    frame.set_ch_layout(ch_layout);
    frame.set_format(sample_fmt);
    frame.set_sample_rate(sample_rate);

    frame
        .get_buffer(0)
        .context("could not allocate output frame samples")?;

    Ok(frame)
}

fn encode_audio_frame(
    mut frame: Option<AVFrame>,
    fmt_ctx_out: &mut AVFormatContextOutput,
    enc_ctx: &mut AVCodecContext,
) -> Result<()> {
    static PTS: AtomicI64 = AtomicI64::new(0);

    if let Some(frame) = frame.as_mut() {
        frame.set_pts(PTS.fetch_add(frame.nb_samples as i64, Ordering::Relaxed));
    }

    enc_ctx.send_frame(frame.as_ref())?;
    loop {
        let mut packet = match enc_ctx.receive_packet() {
            Ok(packet) => packet,
            Err(RsmpegError::EncoderDrainError) | Err(RsmpegError::EncoderFlushedError) => {
                break;
            }
            Err(e) => anyhow::bail!(e),
        };

        fmt_ctx_out
            .write_frame(&mut packet)
            .context("could not write frame")?;
    }
    Ok(())
}

fn load_encode_and_write(
    fifo: &mut AVAudioFifo,
    fmt_ctx_out: &mut AVFormatContextOutput,
    enc_ctx: &mut AVCodecContext,
) -> Result<()> {
    let frame_size = fifo.size().min(enc_ctx.frame_size);

    let mut frame = init_output_frame(
        frame_size,
        enc_ctx.ch_layout().clone().into_inner(),
        enc_ctx.sample_fmt,
        enc_ctx.sample_rate,
    )?;
    if unsafe { fifo.read(frame.data_mut().as_mut_ptr(), frame_size)? } < frame_size {
        anyhow::bail!("Could not read data from FIFO");
    }
    encode_audio_frame(Some(frame), fmt_ctx_out, enc_ctx)?;

    Ok(())
}

pub fn transcode(input_path: &CStr, output_path: &CStr, output_bit_rate: i64) -> Result<()> {
    let (mut fmt_ctx_in, mut dec_ctx, audio_idx) = open_input_file(input_path)?;
    let (mut fmt_ctx_out, mut enc_ctx) = open_output_file(output_path, &dec_ctx, output_bit_rate)?;
    let mut resample_context = init_resampler(&mut dec_ctx, &mut enc_ctx)?;

    // Initialize the FIFO buffer to store audio samples to be encoded.
    let mut fifo = AVAudioFifo::new(enc_ctx.sample_fmt, enc_ctx.ch_layout.nb_channels, 1);

    // Write the header of the output file container.
    fmt_ctx_out
        .write_header(&mut None)
        .context("could not write output file header")?;

    // Loop as long as we have input samples to read or output samples to write.
    // Abort as soon as we have neither.
    loop {
        let output_frame_size = enc_ctx.frame_size;

        loop {
            // We get enough audio samples.
            if fifo.size() >= output_frame_size {
                break;
            }

            // Break when no more input packets.
            let packet = match fmt_ctx_in
                .read_packet()
                .context("could not read input frame")?
            {
                Some(x) => x,
                None => break,
            };

            // Ignore non audio stream packets.
            if packet.stream_index as usize != audio_idx {
                continue;
            }

            dec_ctx
                .send_packet(Some(&packet))
                .context("could not send packet for decoding")?;

            loop {
                let frame = match dec_ctx.receive_frame() {
                    Ok(frame) => frame,
                    Err(RsmpegError::DecoderDrainError) | Err(RsmpegError::DecoderFlushedError) => {
                        break;
                    }
                    Err(e) => anyhow::bail!(e),
                };

                let mut output_samples = AVSamples::new(
                    enc_ctx.ch_layout.nb_channels,
                    frame.nb_samples,
                    enc_ctx.sample_fmt,
                    0,
                )
                .context("could not create samples buffer")?;

                unsafe {
                    resample_context
                        .convert(
                            output_samples.audio_data.as_mut_ptr(),
                            output_samples.nb_samples,
                            frame.extended_data as *const _,
                            frame.nb_samples,
                        )
                        .context("could not convert input samples")?;
                }

                add_samples_to_fifo(&mut fifo, &output_samples, frame.nb_samples)?;
            }
        }

        // If we still cannot get enough samples, break.
        if fifo.size() < output_frame_size {
            break;
        }

        // Write frame as much as possible.
        while fifo.size() >= output_frame_size {
            load_encode_and_write(&mut fifo, &mut fmt_ctx_out, &mut enc_ctx)?;
        }
    }

    // Flush encode context
    encode_audio_frame(None, &mut fmt_ctx_out, &mut enc_ctx)?;
    fmt_ctx_out.write_trailer()?;

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
