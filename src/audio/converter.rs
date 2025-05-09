use anyhow::Result;
use ffmpeg_next as ffmpeg;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::Cursor;

pub struct AudioConverter;

trait AudioFrameWriter {
    fn write_samples(
        &self,
        decoded: &ffmpeg::frame::Audio,
        wav: &mut WavWriter<Cursor<&mut Vec<u8>>>,
    ) -> Result<()>;
}

struct S16Writer;
struct FltWriter;
struct FltpWriter;

impl AudioFrameWriter for S16Writer {
    fn write_samples(
        &self,
        decoded: &ffmpeg::frame::Audio,
        wav: &mut WavWriter<Cursor<&mut Vec<u8>>>,
    ) -> Result<()> {
        for sample_idx in 0..decoded.samples() {
            for channel in 0..decoded.channels() {
                let sample = decoded.plane::<i16>(0)
                    [sample_idx * decoded.channels() as usize + channel as usize];
                wav.write_sample(sample)?;
            }
        }
        Ok(())
    }
}

impl AudioFrameWriter for FltWriter {
    fn write_samples(
        &self,
        decoded: &ffmpeg::frame::Audio,
        wav: &mut WavWriter<Cursor<&mut Vec<u8>>>,
    ) -> Result<()> {
        for sample_idx in 0..decoded.samples() {
            for channel in 0..decoded.channels() {
                let sample = decoded.plane::<f32>(0)
                    [sample_idx * decoded.channels() as usize + channel as usize];
                let scaled = (sample * 32767.0).max(-32768.0).min(32767.0) as i16;
                wav.write_sample(scaled)?;
            }
        }
        Ok(())
    }
}

impl AudioFrameWriter for FltpWriter {
    fn write_samples(
        &self,
        decoded: &ffmpeg::frame::Audio,
        wav: &mut WavWriter<Cursor<&mut Vec<u8>>>,
    ) -> Result<()> {
        let channels = decoded.channels() as usize;
        let samples = decoded.samples() as usize;
        let planes: Vec<&[f32]> = (0..channels).map(|ch| decoded.plane::<f32>(ch)).collect();
        for sample_idx in 0..samples {
            for channel in 0..channels {
                let sample = planes[channel][sample_idx];
                let scaled = (sample * 32767.0).max(-32768.0).min(32767.0) as i16;
                wav.write_sample(scaled)?;
            }
        }
        Ok(())
    }
}

impl AudioConverter {
    pub fn webm_to_wav_bytes(webm_data: &[u8]) -> Result<Vec<u8>> {
        ffmpeg::init()?;

        // Create a temporary file to store the webm data
        let mut temp_file = tempfile::NamedTempFile::new()?;
        std::io::copy(&mut Cursor::new(webm_data), &mut temp_file)?;

        // Create input context from the temporary file
        let mut ictx = ffmpeg::format::input(&temp_file.path())?;

        // Find the audio stream index and decoder
        let (stream_index, mut decoder) = {
            let stream = ictx
                .streams()
                .best(ffmpeg::media::Type::Audio)
                .ok_or_else(|| anyhow::anyhow!("No audio stream found"))?;
            let stream_index = stream.index();
            let decoder_params = stream.parameters();
            let decoder = ffmpeg::codec::Context::from_parameters(decoder_params)?
                .decoder()
                .audio()?;
            (stream_index, decoder)
        };

        // Prepare WAV writer with appropriate specifications
        let spec = WavSpec {
            channels: decoder.channels() as u16,
            sample_rate: decoder.rate() as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut out_buf = Vec::new();
        let mut wav = WavWriter::new(Cursor::new(&mut out_buf), spec)?;
        let mut decoded = ffmpeg::frame::Audio::empty();

        // Select the appropriate writer based on format
        let get_writer = |format_name: &str| -> Result<Box<dyn AudioFrameWriter>> {
            match format_name {
                "s16" => Ok(Box::new(S16Writer)),
                "flt" => Ok(Box::new(FltWriter)),
                "fltp" => Ok(Box::new(FltpWriter)),
                other => Err(anyhow::anyhow!("Unsupported audio format: {}", other)),
            }
        };

        // Process packets and decode frames
        let mut writer: Option<Box<dyn AudioFrameWriter>> = None;
        for (stream_idx, packet) in ictx.packets() {
            if stream_idx.index() != stream_index {
                continue;
            }
            decoder.send_packet(&packet)?;
            while decoder.receive_frame(&mut decoded).is_ok() {
                if writer.is_none() {
                    writer = Some(get_writer(decoded.format().name())?);
                }
                writer.as_ref().unwrap().write_samples(&decoded, &mut wav)?;
            }
        }

        // Flush the decoder
        decoder.send_eof()?;
        while decoder.receive_frame(&mut decoded).is_ok() {
            if writer.is_none() {
                writer = Some(get_writer(decoded.format().name())?);
            }
            writer.as_ref().unwrap().write_samples(&decoded, &mut wav)?;
        }

        wav.finalize()?;
        Ok(out_buf)
    }
}
