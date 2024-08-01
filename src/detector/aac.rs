use crate::detector::{AacDetector, Detector};

impl Detector for AacDetector {
    fn detect(&self, _buffer: &[u8], _offset: usize) -> Option<(usize, usize)> {
        None
    }
}
