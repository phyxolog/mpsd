use super::{DetectOptions, Detector, Mp3Detector, StreamMatch};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum MpegVersion {
    Mpeg1,
    Mpeg2,
    Mpeg2_5,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum MpegLayer {
    Layer1,
    Layer2,
    Layer3,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BitRate {
    Kbps8,
    Kbps16,
    Kbps24,
    Kbps32,
    Kbps40,
    Kbps48,
    Kbps56,
    Kbps64,
    Kbps80,
    Kbps96,
    Kbps112,
    Kbps128,
    Kbps144,
    Kbps160,
    Kbps192,
    Kbps224,
    Kbps256,
    Kbps320,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SampleRate {
    Hz8000,
    Hz11025,
    Hz12000,
    Hz16000,
    Hz22050,
    Hz24000,
    Hz32000,
    Hz44100,
    Hz48000,
}

impl SampleRate {
    pub fn hz(self) -> u32 {
        match self {
            SampleRate::Hz8000 => 8_000,
            SampleRate::Hz11025 => 11_025,
            SampleRate::Hz12000 => 12_000,
            SampleRate::Hz16000 => 16_000,
            SampleRate::Hz22050 => 22_050,
            SampleRate::Hz24000 => 24_000,
            SampleRate::Hz32000 => 32_000,
            SampleRate::Hz44100 => 44_100,
            SampleRate::Hz48000 => 48_000,
        }
    }
}

impl BitRate {
    pub fn bps(self) -> u32 {
        match self {
            BitRate::Kbps8 => 8_000,
            BitRate::Kbps16 => 16_000,
            BitRate::Kbps24 => 24_000,
            BitRate::Kbps32 => 32_000,
            BitRate::Kbps40 => 40_000,
            BitRate::Kbps48 => 48_000,
            BitRate::Kbps56 => 56_000,
            BitRate::Kbps64 => 64_000,
            BitRate::Kbps80 => 80_000,
            BitRate::Kbps96 => 96_000,
            BitRate::Kbps112 => 112_000,
            BitRate::Kbps128 => 128_000,
            BitRate::Kbps144 => 144_000,
            BitRate::Kbps160 => 160_000,
            BitRate::Kbps192 => 192_000,
            BitRate::Kbps224 => 224_000,
            BitRate::Kbps256 => 256_000,
            BitRate::Kbps320 => 320_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameHeader {
    pub layer: MpegLayer,
    pub data_size: usize,
}

fn parse_frame_header(bytes: &[u8]) -> Option<FrameHeader> {
    if bytes[0] != 0xFF && bytes[1] & 0b1110_0000 != 0b1110_0000 {
        return None;
    }

    let crc = bytes[1] & 1 == 0;

    let version = match bytes[1] & 0b0001_1000 {
        0b00_000 => MpegVersion::Mpeg2_5,
        0b10_000 => MpegVersion::Mpeg2,
        0b11_000 => MpegVersion::Mpeg1,
        _ => return None,
    };

    let layer = match bytes[1] & 0b110 {
        0b010 => MpegLayer::Layer3,
        0b100 => MpegLayer::Layer2,
        0b110 => MpegLayer::Layer1,
        _ => return None,
    };

    let is_version2 = version == MpegVersion::Mpeg2 || version == MpegVersion::Mpeg2_5;

    let bitrate = match (bytes[2] & 0b1111_0000, is_version2) {
        (0b0001_0000, false) => BitRate::Kbps32,
        (0b0010_0000, false) => BitRate::Kbps40,
        (0b0011_0000, false) => BitRate::Kbps48,
        (0b0100_0000, false) => BitRate::Kbps56,
        (0b0101_0000, false) => BitRate::Kbps64,
        (0b0110_0000, false) => BitRate::Kbps80,
        (0b0111_0000, false) => BitRate::Kbps96,
        (0b1000_0000, false) => BitRate::Kbps112,
        (0b1001_0000, false) => BitRate::Kbps128,
        (0b1010_0000, false) => BitRate::Kbps160,
        (0b1011_0000, false) => BitRate::Kbps192,
        (0b1100_0000, false) => BitRate::Kbps224,
        (0b1101_0000, false) => BitRate::Kbps256,
        (0b1110_0000, false) => BitRate::Kbps320,

        (0b0001_0000, true) => BitRate::Kbps8,
        (0b0010_0000, true) => BitRate::Kbps16,
        (0b0011_0000, true) => BitRate::Kbps24,
        (0b0100_0000, true) => BitRate::Kbps32,
        (0b0101_0000, true) => BitRate::Kbps40,
        (0b0110_0000, true) => BitRate::Kbps48,
        (0b0111_0000, true) => BitRate::Kbps56,
        (0b1000_0000, true) => BitRate::Kbps64,
        (0b1001_0000, true) => BitRate::Kbps80,
        (0b1010_0000, true) => BitRate::Kbps96,
        (0b1011_0000, true) => BitRate::Kbps112,
        (0b1100_0000, true) => BitRate::Kbps128,
        (0b1101_0000, true) => BitRate::Kbps144,
        (0b1110_0000, true) => BitRate::Kbps160,
        _ => return None,
    };

    let padding = bytes[2] & 0b10 != 0;

    let sample_rate = match (bytes[2] & 0b0000_1100, version) {
        (0b00_00, MpegVersion::Mpeg1) => SampleRate::Hz44100,
        (0b00_00, MpegVersion::Mpeg2) => SampleRate::Hz22050,
        (0b00_00, MpegVersion::Mpeg2_5) => SampleRate::Hz11025,
        (0b01_00, MpegVersion::Mpeg1) => SampleRate::Hz48000,
        (0b01_00, MpegVersion::Mpeg2) => SampleRate::Hz24000,
        (0b01_00, MpegVersion::Mpeg2_5) => SampleRate::Hz12000,
        (0b10_00, MpegVersion::Mpeg1) => SampleRate::Hz32000,
        (0b10_00, MpegVersion::Mpeg2) => SampleRate::Hz16000,
        (0b10_00, MpegVersion::Mpeg2_5) => SampleRate::Hz8000,
        _ => return None,
    };

    let bits_per_sample = match version {
        MpegVersion::Mpeg1 => 144,
        MpegVersion::Mpeg2 => 72,
        MpegVersion::Mpeg2_5 => 72,
    };

    let data_size = (bits_per_sample * bitrate.bps() / sample_rate.hz()
        + if padding { 1 } else { 0 }
        - if crc { 2 } else { 0 }) as usize;

    Some(FrameHeader { layer, data_size })
}

impl Detector for Mp3Detector {
    fn detect(&self, buffer: &[u8], offset: usize, opts: &DetectOptions) -> Option<StreamMatch> {
        let mut offset2 = offset;
        let mut size: usize = 0;
        let mut frames: usize = 0;
        let mut layer: MpegLayer = MpegLayer::Layer3;

        loop {
            if offset2 + 3 > buffer.len() {
                break;
            }

            if opts.mpeg_max_frames != 0 && frames >= opts.mpeg_max_frames as usize {
                break;
            }

            let bytes = &buffer[offset2..offset2 + 3];

            if let Some(frame_header) = parse_frame_header(bytes) {
                if frames > 0 && frame_header.layer != layer {
                    break;
                }

                layer = frame_header.layer;
                size += frame_header.data_size;
                offset2 += frame_header.data_size;
                frames += 1;
            } else {
                break;
            }
        }

        if opts.mpeg_min_frames != 0 && frames <= opts.mpeg_min_frames as usize {
            return None;
        }

        let ext = match layer {
            MpegLayer::Layer1 => "mp1",
            MpegLayer::Layer2 => "mp2",
            MpegLayer::Layer3 => "mp3",
        };

        if size > 0 {
            return Some(StreamMatch { offset, size, ext });
        }

        return None;
    }
}
