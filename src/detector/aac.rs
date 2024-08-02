use crate::detector::{AacDetector, DetectOptions, Detector};

impl Detector for AacDetector {
    fn detect(
        &self,
        _buffer: &[u8],
        _offset: usize,
        _opts: &DetectOptions,
    ) -> Option<(usize, usize)> {
        None
    }
}
