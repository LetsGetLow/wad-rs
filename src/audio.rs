/// A structure representing a sound sample with its sample rate and audio data.
/// The audio data is stored as a vector of f32 samples normalized between -1.0 and 1.0.
///
/// # Format
/// The sound sample can be created from a byte slice that follows a specific format:
/// - 8 bytes header followed by audio sample data as 8-bit unsigned integers.
///
/// # Header Format
/// - The first 2 bytes represent the magic number (u16, little-endian, always 768).
/// - The next 2 bytes represent the sample rate (u16, little-endian).
/// - The next 4 bytes represent the number of samples (u32, little-endian).
#[derive(Debug, Clone)]
pub struct SoundSample {
    sample_rate: u32,
    sample: Vec<f32>,
}

impl SoundSample {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn sample(&self) -> &[f32] {
        &self.sample
    }

    pub fn is_sound_sample(data: &[u8]) -> bool {
        data.starts_with(&[0x03, 0x00])
    }
}

impl TryFrom<&[u8]> for SoundSample {
    type Error = Box<dyn std::error::Error>;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 8 {
            return Err("Data too short to contain valid sound sample header".into());
        }

        if data[0] != 0x03 || data[1] != 0x00 {
            return Err("Invalid sound sample magic number".into());
        }

        let sample_rate = u16::from_le_bytes([data[2], data[3]]) as u32;
        let sample_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let sample_end = 8 + sample_count;
        if sample_end > data.len() {
            return Err("Data too short to contain declared number of samples".into());
        }

        let sample = data[8..sample_end]
            .iter()
            .map(|&b| (b as f32 - 128.0) / 128.0)
            .collect();

        Ok(SoundSample {
            sample_rate,
            sample,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicType {
    Mus,
    Midi,
    Unknown,
}

/// A structure representing a music file.
#[derive(Debug, Clone)]
pub struct MusicSample {
}

impl MusicSample {
    pub fn determine_type(data: &[u8]) -> MusicType {
        match data.get(..4) {
            Some(b"MUS\x1A") => MusicType::Mus,
            Some(b"MThd") => MusicType::Midi,
            _ => MusicType::Unknown,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_sample_conversion_fails_on_to_short_data() {
        let data = vec![0u8; 4];
        let result = SoundSample::try_from(data.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn sound_sample_conversion_fails_on_invalid_magic_number() {
        let data = vec![0u8; 10];
        let result = SoundSample::try_from(data.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn sound_sample_conversion_fails_on_invalid_sample_count() {
        let data = vec![
            0x03, 0x00, // Magic number
            0x40, 0x1F, // Sample rate (8000)
            0xFF, 0xFF, 0xFF, 0xFF, // Sample count (4294967295)
        ];
        let result = SoundSample::try_from(data.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn sound_sample_conversion_succeeds_on_valid_data() {
        let data = vec![
            0x03, 0x00, // Magic number
            0x40, 0x1F, // Sample rate (8000)
            0x04, 0x00, 0x00, 0x00, // Sample count (4)
            0x00, 0x80, 0xFF, 0x7F, // Sample data
        ];
        let result = SoundSample::try_from(data.as_slice());
        assert!(result.is_ok());
        let sound_sample = result.unwrap();
        assert_eq!(sound_sample.sample_rate(), 8000);
        assert_eq!(sound_sample.sample(), &[-1.0, 0.0, 0.9921875, -0.0078125]);
    }

    #[test]
    fn sound_sample_detects_valid_magic_number() {
        let valid_magic = [0x03, 0x00];
        let invalid_magic = [0x04, 0x00];
        assert!(SoundSample::is_sound_sample(&valid_magic));
        assert!(!SoundSample::is_sound_sample(&invalid_magic));
    }

    #[test]
    fn music_sample_detects_types() {
        let mus_data = b"MUS\x1Arest of the data";
        let midi_data = b"MThdrest of the data";
        let unknown_data = b"XXXXrest of the data";
        let too_short_data = b"MU";

        assert_eq!(MusicSample::determine_type(mus_data), MusicType::Mus);
        assert_eq!(MusicSample::determine_type(midi_data), MusicType::Midi);
        assert_eq!(MusicSample::determine_type(unknown_data), MusicType::Unknown);
        assert_eq!(MusicSample::determine_type(too_short_data), MusicType::Unknown);
    }
}
