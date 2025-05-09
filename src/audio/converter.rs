use anyhow::{anyhow, Result};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::{codec, ffi, format, frame, media};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::Cursor;
use std::io::Read as IoRead;
use std::ptr;
pub fn webm_to_wav_bytes(data: &[u8]) -> Result<Vec<u8>> {
    ffmpeg::init()?;

    /* ---------- 1. custom AVIO that streams from the slice ---------- */
    const BUF: usize = 4 * 1024;
    unsafe extern "C" fn read(
        opaque: *mut libc::c_void,
        buf: *mut u8,
        buf_size: libc::c_int,
    ) -> libc::c_int {
        let cursor = &mut *(opaque as *mut Cursor<&[u8]>);
        let out = std::slice::from_raw_parts_mut(buf, buf_size as usize);
        match IoRead::read(cursor, out) {
            Ok(0) => ffi::AVERROR_EOF,
            Ok(n) => n as libc::c_int,
            Err(_) => -1,
        }
    }

    let cursor = Box::new(Cursor::new(data));
    let avio_buf = unsafe { ffi::av_malloc(BUF) } as *mut u8;
    if avio_buf.is_null() {
        return Err(anyhow!("av_malloc failed"));
    }

    let avio = unsafe {
        ffi::avio_alloc_context(
            avio_buf,
            BUF as i32,
            0,
            Box::into_raw(cursor) as *mut _,
            Some(read),
            None,
            None,
        )
    };
    if avio.is_null() {
        return Err(anyhow!("avio_alloc_context failed"));
    }

    /* ---------- 2. open demuxer on that AVIO ---------- */
    let mut ictx = unsafe {
        let mut ctx = ffi::avformat_alloc_context();
        (*ctx).pb = avio;
        (*ctx).flags |= ffi::AVFMT_FLAG_CUSTOM_IO;
        if ffi::avformat_open_input(&mut ctx, ptr::null(), ptr::null_mut(), ptr::null_mut()) < 0 {
            return Err(anyhow!("avformat_open_input failed"));
        }
        format::context::Input::wrap(ctx)
    };

    /* ---------- 3. find decoder ---------- */
    let (sidx, mut dec) = {
        let s = ictx
            .streams()
            .best(media::Type::Audio)
            .ok_or_else(|| anyhow!("no audio stream"))?;
        let idx = s.index();
        let d = codec::Context::from_parameters(s.parameters())?
            .decoder()
            .audio()?;
        (idx, d)
    };

    /* ---------- 4. set up WAV writer ---------- */
    let spec = WavSpec {
        channels: dec.channels(),
        sample_rate: dec.rate(),
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut out = Vec::new();
    let mut wav = WavWriter::new(Cursor::new(&mut out), spec)?;
    let mut frm = frame::Audio::empty();

    /* helpers for s16 / flt */
    trait Writer {
        fn write(&self, f: &frame::Audio, w: &mut WavWriter<Cursor<&mut Vec<u8>>>) -> Result<()>;
    }
    struct S16;
    struct F32;
    impl Writer for S16 {
        fn write(
            &self,
            f: &frame::Audio,
            w: &mut WavWriter<Cursor<&mut Vec<u8>>>,
        ) -> anyhow::Result<()> {
            // s16 in WebM is usually *packed* ⇒ single plane (index 0)
            for &s in f.plane::<i16>(0) {
                w.write_sample(s)?;
            }
            Ok(())
        }
    }

    impl Writer for F32 {
        fn write(
            &self,
            f: &frame::Audio,
            w: &mut WavWriter<Cursor<&mut Vec<u8>>>,
        ) -> anyhow::Result<()> {
            if f.format().is_planar() {
                // fltp ⇒ one plane per channel
                for ch in 0..f.channels() {
                    for &s in f.plane::<f32>(ch as usize) {
                        w.write_sample((s * i16::MAX as f32) as i16)?;
                    }
                }
            } else {
                // packed float (rare in WebM) ⇒ single plane
                for &s in f.plane::<f32>(0) {
                    w.write_sample((s * i16::MAX as f32) as i16)?;
                }
            }
            Ok(())
        }
    }

    /* ---------- 5. decode & write ---------- */
    let mut wr: Option<Box<dyn Writer>> = None;
    for (idx, pkt) in ictx.packets() {
        if idx.index() != sidx {
            continue;
        }
        dec.send_packet(&pkt)?;
        while dec.receive_frame(&mut frm).is_ok() {
            if wr.is_none() {
                wr = Some(match frm.format().name() {
                    "s16" => Box::new(S16),
                    "flt" | "fltp" => Box::new(F32),
                    f => return Err(anyhow!("unsupported {}", f)),
                });
            }
            wr.as_ref().unwrap().write(&frm, &mut wav)?;
        }
    }
    dec.send_eof()?;
    while dec.receive_frame(&mut frm).is_ok() {
        wr.as_ref().unwrap().write(&frm, &mut wav)?;
    }
    wav.finalize()?;

    /* ---------- 6. cleanup ---------- */
    unsafe {
        ffi::avio_context_free(&mut (avio as *mut _));
    }
    Ok(out)
}
