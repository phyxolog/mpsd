pub mod aac;
pub mod bitmap;
pub mod mp3;
pub mod ogg;
pub mod riff_wave;

pub struct DetectOptions {
    pub mpeg_min_frames: u8,
    pub mpeg_max_frames: u16,
}

pub struct StreamMatch<'a> {
    pub offset: usize,
    pub size: usize,
    pub ext: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub enum StreamType {
    RiffWave,
    Bitmap,
    Ogg,
    Aac,
    Mp3,
}

pub trait Detector {
    fn detect(&self, buffer: &[u8], offset: usize, opts: &DetectOptions) -> Option<StreamMatch>;
}

pub struct RiffWaveDetector;
pub struct BitmapDetector;
pub struct OggDetector;
pub struct AacDetector;
pub struct Mp3Detector;
