use crate::detector::{Detector, RiffWaveDetector};

#[repr(C, packed)]
#[derive(Debug, Default)]
struct RiffWaveHeader {
    chunk_id: [u8; 4],
    chunk_size: u32,
    format: [u8; 4],
    subchunk1_id: [u8; 4],
    subchunk1_size: u32,
    audio_format: u16,
    num_channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
    subchunk2_id: [u8; 4],
    subchunk2_size: u32,
}

impl Detector for RiffWaveDetector {
    fn detect(&self, buffer: &[u8], offset: usize) -> Option<(usize, usize)> {
        if offset + std::mem::size_of::<RiffWaveHeader>() > buffer.len() {
            return None;
        }

        let (head, body, _tail) = unsafe {
            &buffer[offset..offset + std::mem::size_of::<RiffWaveHeader>()]
                .align_to::<RiffWaveHeader>()
        };

        if !head.is_empty() {
            return None;
        }

        let header = &body[0];

        if &header.chunk_id != b"RIFF" {
            return None;
        }

        if &header.format != b"WAVE" {
            return None;
        }

        if &header.subchunk1_id != b"fmt " {
            return None;
        }

        if &header.subchunk2_id != b"data" {
            return None;
        }

        if header.audio_format != 1 {
            return None;
        }

        if header.subchunk1_size != 16 {
            return None;
        }

        if header.byte_rate
            != header.sample_rate * header.num_channels as u32 * header.bits_per_sample as u32 / 8
        {
            return None;
        }

        if header.block_align != header.num_channels * header.bits_per_sample / 8 {
            return None;
        }

        let data_size: usize =
            usize::try_from(header.subchunk2_size).unwrap() + std::mem::size_of::<RiffWaveHeader>();

        let chunk_size: usize = usize::try_from(header.chunk_size).unwrap() + 8;

        let min = chunk_size.min(data_size) as f64;
        let max = chunk_size.max(data_size) as f64;

        // if the difference between the chunk size and the data size is 5% or more,
        // we consider this to be an incorrect header
        if ((max - min) / max) * 100.0 >= 5.0 {
            return None;
        }

        // data_size is more directly relevant because it indicates the amount
        // of actual audio data
        if offset + data_size > buffer.len() {
            return None;
        }

        return Some((offset, data_size));
    }
}
