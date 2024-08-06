use super::{DetectOptions, Detector, RiffWaveDetector, StreamMatch};
use std::mem::size_of;

#[repr(C, packed)]
#[derive(Debug, Default)]
struct RiffHeader {
    id: [u8; 4],
    chunk_size: u32,
    format: [u8; 4],
}

#[repr(C, packed)]
#[derive(Debug, Default)]
struct WaveChunk {
    id: [u8; 4],
    size: u32,
}

#[repr(C, packed)]
#[derive(Debug, Default)]
struct WaveFormat {
    format_tag: u16,
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
}

#[repr(C, packed)]
#[derive(Default)]
struct WaveFormatPCM {
    format: WaveFormat,
    bits_per_sample: u16,
}

#[repr(C, packed)]
#[derive(Default)]
struct RiffWavePCMHeader {
    header: RiffHeader,
    chunk1: WaveChunk,
    pcm_format: WaveFormatPCM,
    chunk2: WaveChunk,
}

impl Detector for RiffWaveDetector {
    fn detect(&self, buffer: &[u8], offset: usize, _opts: &DetectOptions) -> Option<StreamMatch> {
        if offset + size_of::<RiffWavePCMHeader>() > buffer.len() {
            return None;
        }

        let (head, body, _tail) = unsafe {
            &buffer[offset..offset + size_of::<RiffWavePCMHeader>()].align_to::<RiffWavePCMHeader>()
        };

        if !head.is_empty() {
            return None;
        }

        let data = &body[0];

        if &data.header.id != b"RIFF" {
            return None;
        }

        if &data.header.format != b"WAVE" {
            return None;
        }

        if &data.chunk1.id != b"fmt " {
            return None;
        }

        if &data.chunk2.id != b"data" {
            return None;
        }

        if data.pcm_format.format.format_tag != 1 {
            return None;
        }

        if data.chunk1.size != 16 {
            return None;
        }

        if data.pcm_format.format.byte_rate
            != data.pcm_format.format.sample_rate
                * data.pcm_format.format.channels as u32
                * data.pcm_format.bits_per_sample as u32
                / 8
        {
            return None;
        }

        if data.pcm_format.format.block_align
            != data.pcm_format.format.channels * data.pcm_format.bits_per_sample / 8
        {
            return None;
        }

        let mut data_size = usize::try_from(data.chunk2.size).unwrap();
        let chunk_size = usize::try_from(data.header.chunk_size).unwrap();

        if chunk_size < data_size {
            return None;
        }

        data_size += size_of::<RiffWavePCMHeader>();

        if offset + data_size > buffer.len() {
            data_size = buffer.len() - offset;
        }

        return Some(StreamMatch {
            offset,
            size: data_size,
            ext: "wav",
        });
    }
}
