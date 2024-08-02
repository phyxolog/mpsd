use crate::detector::{BitmapDetector, DetectOptions, Detector};

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

impl Detector for BitmapDetector {
    fn detect(
        &self,
        buffer: &[u8],
        offset: usize,
        _opts: &DetectOptions,
    ) -> Option<(usize, usize)> {
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
