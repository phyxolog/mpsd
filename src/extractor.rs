use std::path::PathBuf;

use crate::detector::StreamType;

pub fn extract(
    _buffer: &[u8],
    _offset: usize,
    _size: usize,
    _stream_type: &StreamType,
    _ext: &str,
    _output_dir: &PathBuf,
) -> usize {
    0
}
