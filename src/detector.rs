use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum StreamType {
    RiffWave,
    Bitmap,
    Ogg,
    Aac,
}

pub trait Detector {
    fn detect(&self, buffer: &[u8], offset: usize) -> Option<(usize, usize)>;
}

pub struct RiffWaveDetector;
pub struct BitmapDetector;
pub struct OggDetector;
pub struct AacDetector;

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

#[repr(C, packed)]
#[derive(Debug, Default)]
struct BitmapHeader {
    id: [u8; 2],
    size: u32,
    reserved1: u16,
    reserved2: u16,
    offset: u32,
    id2: u32,
    width: u32,
    height: u32,
    planes: u16,
    bpp: u16,
    compression: u32,
    size_image: u32,
    xppm: u32,
    yppm: u32,
    colors_used: u32,
    colors_important: u32,
}

#[repr(C, packed)]
#[derive(Debug, Default)]
struct OggHeader {
    id: [u8; 4],
    revision: u8,
    bit_flags: u8,
    absolute_granule_pos: i64,
    stream_serial_number: u32,
    page_seq_num: u32,
    page_checksum: u32,
    num_page_segments: u8,
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

        // let real_size = 4 + (8 + header.subchunk1_size) + (8 + header.subchunk2_size);
        let size: usize = usize::try_from(header.chunk_size + 8).unwrap();

        if offset + size > buffer.len() {
            return None;
        }

        return Some((offset, size));
    }
}

impl Detector for BitmapDetector {
    fn detect(&self, buffer: &[u8], offset: usize) -> Option<(usize, usize)> {
        if offset + std::mem::size_of::<BitmapHeader>() > buffer.len() {
            return None;
        }

        let (head, body, _tail) = unsafe {
            &buffer[offset..offset + std::mem::size_of::<BitmapHeader>()].align_to::<BitmapHeader>()
        };

        if !head.is_empty() {
            return None;
        }

        let header = &body[0];

        if &header.id != b"BM" {
            return None;
        }

        if header.id2 != 40 {
            return None;
        }

        if header.size < 14 {
            return None;
        }

        let size: usize = usize::try_from(header.size).unwrap();

        if offset + size > buffer.len() {
            return None;
        }

        return Some((offset, size));
    }
}

impl Detector for OggDetector {
    fn detect(&self, buffer: &[u8], offset: usize) -> Option<(usize, usize)> {
        let mut size = 0;
        let mut offset2 = offset;
        let mut first_occurrence = true;

        loop {
            if offset2 + std::mem::size_of::<OggHeader>() > buffer.len() {
                break;
            }

            let (head, body, _tail) = unsafe {
                &buffer[offset2..offset2 + std::mem::size_of::<OggHeader>()].align_to::<OggHeader>()
            };

            offset2 += std::mem::size_of::<OggHeader>();

            if !head.is_empty() {
                break;
            }

            let header = &body[0];

            if &header.id != b"OggS" && header.revision != 0 {
                break;
            }

            if header.bit_flags & 2 != if first_occurrence { 2 } else { 0 } {
                break;
            }

            let end = offset2 + header.num_page_segments as usize;

            if end > buffer.len() {
                break;
            }

            for &byte in &buffer[offset2..end] {
                size += byte as usize;
            }

            size += std::mem::size_of::<OggHeader>() + header.num_page_segments as usize;

            if (header.bit_flags & 4) == 4 {
                break;
            }

            offset2 = offset + size as usize;
            first_occurrence = false;
        }

        if size == 0 {
            return None;
        }

        if offset + size > buffer.len() {
            return None;
        }

        return Some((offset, size));
    }
}

impl Detector for AacDetector {
    fn detect(&self, buffer: &[u8], offset: usize) -> Option<(usize, usize)> {
        // if (buffer[offset + 1] & 240) == 240 {
        //     return Some((offset, 12));
        // }

        None
    }
}
