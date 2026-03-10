use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputMediaInfo {
    pub path: String,
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub channel_layout: String,
    pub codec_name: String,
}

pub fn probe_audio_inputs(
    storage_path: &str,
    file_names: &[String],
) -> Result<Vec<InputMediaInfo>> {
    file_names
        .iter()
        .map(|file_name| {
            let path = PathBuf::from(storage_path).join(file_name);
            probe_audio_file(&path)
        })
        .collect()
}

pub fn validate_consistent_audio_inputs(inputs: &[InputMediaInfo]) -> Result<()> {
    let Some(first) = inputs.first() else {
        return Ok(());
    };

    for info in &inputs[1..] {
        if info.channels != first.channels
            || info.sample_rate != first.sample_rate
            || info.bits_per_sample != first.bits_per_sample
        {
            bail!(
                "album mode requires consistent audio input media; expected channels/sample_rate/bits_per_sample = {}/{}/{}, got {}/{}/{} for {}",
                first.channels,
                first.sample_rate,
                first.bits_per_sample,
                info.channels,
                info.sample_rate,
                info.bits_per_sample,
                info.path
            );
        }
    }

    Ok(())
}

fn probe_audio_file(path: &Path) -> Result<InputMediaInfo> {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    if ext != "wav" {
        bail!(
            "unsupported input audio format for {}; only wav/bwf/adm-wav inputs are currently supported",
            path.display()
        );
    }

    probe_wav_file(path)
}

fn probe_wav_file(path: &Path) -> Result<InputMediaInfo> {
    let mut file = File::open(path)
        .with_context(|| format!("failed to open input audio {}", path.display()))?;

    let mut riff_header = [0_u8; 12];
    file.read_exact(&mut riff_header)
        .with_context(|| format!("failed to read RIFF header from {}", path.display()))?;

    if &riff_header[0..4] != b"RIFF" || &riff_header[8..12] != b"WAVE" {
        bail!(
            "unsupported wav container in {}; expected RIFF/WAVE",
            path.display()
        );
    }

    let mut fmt_chunk = None;
    loop {
        let mut chunk_header = [0_u8; 8];
        match file.read_exact(&mut chunk_header) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("failed to read WAV chunk header from {}", path.display())
                });
            }
        }

        let chunk_id = &chunk_header[0..4];
        let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().expect("u32 bytes"));

        if chunk_id == b"fmt " {
            let mut data = vec![0_u8; chunk_size as usize];
            file.read_exact(&mut data)
                .with_context(|| format!("failed to read WAV fmt chunk from {}", path.display()))?;
            fmt_chunk = Some(parse_fmt_chunk(path, &data)?);
        } else {
            file.seek(SeekFrom::Current(i64::from(chunk_size)))
                .with_context(|| format!("failed to skip WAV chunk in {}", path.display()))?;
        }

        if chunk_size % 2 == 1 {
            file.seek(SeekFrom::Current(1)).with_context(|| {
                format!("failed to align WAV chunk cursor in {}", path.display())
            })?;
        }
    }

    let fmt =
        fmt_chunk.ok_or_else(|| anyhow::anyhow!("missing WAV fmt chunk in {}", path.display()))?;
    Ok(InputMediaInfo {
        path: path.display().to_string(),
        channels: fmt.channels,
        sample_rate: fmt.sample_rate,
        bits_per_sample: fmt.bits_per_sample,
        channel_layout: fmt.channel_layout,
        codec_name: fmt.codec_name,
    })
}

struct ParsedFmt {
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
    channel_layout: String,
    codec_name: String,
}

fn parse_fmt_chunk(path: &Path, data: &[u8]) -> Result<ParsedFmt> {
    if data.len() < 16 {
        bail!(
            "invalid WAV fmt chunk in {}; expected at least 16 bytes",
            path.display()
        );
    }

    let format_tag = u16::from_le_bytes(data[0..2].try_into().expect("u16 bytes"));
    let channels = u16::from_le_bytes(data[2..4].try_into().expect("u16 bytes"));
    let sample_rate = u32::from_le_bytes(data[4..8].try_into().expect("u32 bytes"));
    let bits_per_sample = u16::from_le_bytes(data[14..16].try_into().expect("u16 bytes"));

    let (channel_layout, codec_name) = if format_tag == 0xFFFE && data.len() >= 40 {
        let channel_mask = u32::from_le_bytes(data[20..24].try_into().expect("u32 bytes"));
        let sub_format = u16::from_le_bytes(data[24..26].try_into().expect("u16 bytes"));
        (
            infer_channel_layout(channels, Some(channel_mask)),
            infer_codec_name(sub_format, bits_per_sample),
        )
    } else {
        (
            infer_channel_layout(channels, None),
            infer_codec_name(format_tag, bits_per_sample),
        )
    };

    Ok(ParsedFmt {
        channels,
        sample_rate,
        bits_per_sample,
        channel_layout,
        codec_name,
    })
}

fn infer_channel_layout(channels: u16, channel_mask: Option<u32>) -> String {
    if let Some(mask) = channel_mask {
        let layout = match mask {
            0x0000_0004 => Some("mono"),
            0x0000_0003 => Some("stereo"),
            0x0000_003F => Some("5.1"),
            0x0000_063F => Some("7.1"),
            _ => None,
        };
        if let Some(layout) = layout {
            return layout.to_string();
        }
    }

    match channels {
        1 => "mono".to_string(),
        2 => "stereo".to_string(),
        6 => "5.1".to_string(),
        8 => "7.1".to_string(),
        _ => "unknown".to_string(),
    }
}

fn infer_codec_name(format_tag: u16, bits_per_sample: u16) -> String {
    match (format_tag, bits_per_sample) {
        (1, 16) => "pcm_s16le".to_string(),
        (1, 24) => "pcm_s24le".to_string(),
        (1, 32) => "pcm_s32le".to_string(),
        (1, bits) => format!("pcm_s{bits}le"),
        (3, 32) => "pcm_f32le".to_string(),
        (3, 64) => "pcm_f64le".to_string(),
        (3, bits) => format!("pcm_f{bits}le"),
        (tag, bits) => format!("format_{tag:#06x}_{bits}bit"),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::Path};

    use tempfile::TempDir;

    use super::{probe_audio_inputs, validate_consistent_audio_inputs};

    fn write_test_wav(path: &Path, channels: u16, bits_per_sample: u16) {
        let sample_rate = 48_000_u32;
        let data_size = u32::from(channels) * u32::from(bits_per_sample / 8) * 8;
        let byte_rate = sample_rate * u32::from(channels) * u32::from(bits_per_sample / 8);
        let block_align = channels * (bits_per_sample / 8);
        let riff_size = 36 + data_size;

        let mut file = File::create(path).expect("create wav");
        file.write_all(b"RIFF").expect("riff");
        file.write_all(&riff_size.to_le_bytes()).expect("riff size");
        file.write_all(b"WAVE").expect("wave");
        file.write_all(b"fmt ").expect("fmt");
        file.write_all(&16_u32.to_le_bytes()).expect("fmt size");
        file.write_all(&1_u16.to_le_bytes()).expect("pcm");
        file.write_all(&channels.to_le_bytes()).expect("channels");
        file.write_all(&sample_rate.to_le_bytes())
            .expect("sample_rate");
        file.write_all(&byte_rate.to_le_bytes()).expect("byte_rate");
        file.write_all(&block_align.to_le_bytes())
            .expect("block_align");
        file.write_all(&bits_per_sample.to_le_bytes())
            .expect("bits_per_sample");
        file.write_all(b"data").expect("data");
        file.write_all(&data_size.to_le_bytes()).expect("data size");
        file.write_all(&vec![0_u8; data_size as usize])
            .expect("samples");
    }

    #[test]
    fn probes_basic_wav_metadata() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("input.wav");
        write_test_wav(&path, 6, 16);

        let infos = probe_audio_inputs(
            temp.path().to_str().expect("utf-8"),
            &[String::from("input.wav")],
        )
        .expect("probe wav");

        assert_eq!(infos.len(), 1);
        let info = &infos[0];
        assert_eq!(info.channels, 6);
        assert_eq!(info.sample_rate, 48_000);
        assert_eq!(info.bits_per_sample, 16);
        assert_eq!(info.channel_layout, "5.1");
        assert_eq!(info.codec_name, "pcm_s16le");
    }

    #[test]
    fn rejects_inconsistent_album_inputs() {
        let temp = TempDir::new().expect("tempdir");
        write_test_wav(&temp.path().join("a.wav"), 6, 16);
        write_test_wav(&temp.path().join("b.wav"), 8, 16);

        let infos = probe_audio_inputs(
            temp.path().to_str().expect("utf-8"),
            &[String::from("a.wav"), String::from("b.wav")],
        )
        .expect("probe wavs");

        let err = validate_consistent_audio_inputs(&infos)
            .expect_err("mismatched channels should fail")
            .to_string();
        assert!(err.contains("album mode requires consistent audio input media"));
        assert!(err.contains("6/48000/16"));
        assert!(err.contains("8/48000/16"));
    }
}
