use crate::detector::{Detector, OggDetector};

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
