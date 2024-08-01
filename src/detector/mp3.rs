use crate::detector::{Detector, Mp3Detector};

impl Detector for Mp3Detector {
    fn detect(&self, _buffer: &[u8], _offset: usize) -> Option<(usize, usize)> {
        None
    }
}
