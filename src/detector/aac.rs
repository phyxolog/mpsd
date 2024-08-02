use super::{AacDetector, DetectOptions, Detector, StreamMatch};

fn is_valid_frame_header(bytes: &[u8]) -> bool {
    bytes[0] == 0xFF
        && (bytes[1] & 0xF0) == 0xF0
        && (bytes[1] & 0x06) == 0x00
        && (bytes[2] & 0x02) == 0x00
        && (bytes[3] & 0x3C) == 0x00
        && (bytes[5] & 0x1F) == 0x1F
        && (bytes[6] & 0xFC) == 0xFC
        && (bytes[6] & 0x03) == 0x00
}

fn parse_frame_length(bytes: &[u8]) -> u16 {
    if !is_valid_frame_header(bytes) {
        return 0;
    }

    let mut frame_length = (bytes[3] & 3) as u16;
    frame_length <<= 11;
    frame_length |= (bytes[4] as u16) << 3;
    frame_length |= ((bytes[5] & 0xE0) >> 5) as u16;

    return frame_length;
}

impl Detector for AacDetector {
    fn detect(&self, buffer: &[u8], offset: usize, opts: &DetectOptions) -> Option<StreamMatch> {
        let mut offset2 = offset;
        let mut size: usize = 0;
        let mut frames: usize = 0;

        loop {
            if offset + size >= buffer.len() {
                break;
            }

            if offset2 + 7 > buffer.len() {
                break;
            }

            if opts.mpeg_max_frames != 0 && frames >= opts.mpeg_max_frames as usize {
                break;
            }

            let bytes = &buffer[offset2..offset2 + 7];
            let frame_length: usize = parse_frame_length(bytes) as usize;

            if frame_length > 0 {
                frames += 1;
                size += frame_length;
                offset2 += frame_length;
            } else {
                break;
            }
        }

        if opts.mpeg_min_frames != 0 && frames <= opts.mpeg_min_frames as usize {
            return None;
        }

        if size > 0 {
            return Some(StreamMatch {
                offset,
                size,
                ext: "aac",
            });
        }

        return None;
    }
}
