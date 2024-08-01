pub mod aac;
pub mod bitmap;
pub mod mp3;
pub mod ogg;
pub mod riff_wave;

#[derive(Debug, Clone, Copy)]
pub enum StreamType {
    RiffWave,
    Bitmap,
    Ogg,
    Aac,
    Mp3,
}

pub trait Detector {
    fn detect(&self, buffer: &[u8], offset: usize) -> Option<(usize, usize)>;
}

pub struct RiffWaveDetector;
pub struct BitmapDetector;
pub struct OggDetector;
pub struct AacDetector;
pub struct Mp3Detector;
