use super::{AacDetector, DetectOptions, Detector, StreamMatch};

impl Detector for AacDetector {
    fn detect(&self, _buffer: &[u8], _offset: usize, _opts: &DetectOptions) -> Option<StreamMatch> {
        None
    }
}
