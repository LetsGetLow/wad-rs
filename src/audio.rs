pub struct SoundSample {
    pub sample_rate: u32,
    pub sample: Vec<f32>,
}

impl SoundSample {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn sample(&self) -> &[f32] {
        &self.sample
    }
}

impl TryFrom<&[u8]> for SoundSample {
    type Error = Box<dyn std::error::Error>;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 8 {
            return Err("Data too short to contain valid sound sample header".into());
        }

        let magic_number = u16::from_be_bytes([data[0], data[1]]);
        if magic_number != 768 {
            return Err("Invalid sound sample magic number".into());
        }

        let sample_rate = u16::from_le_bytes([data[2], data[3]]) as u32;
        let sample_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let sample_end = 8 + sample_count;
        if sample_end > data.len() {
            return Err("Data too short to contain declared number of samples".into());
        }

        let mut sample = Vec::with_capacity(sample_count);
        sample = data[8..sample_end]
            .iter()
            .map(|&b| (b as f32 - 128.0) / 128.0)
            .collect();

        Ok(SoundSample {
            sample_rate,
            sample,
        })
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
}