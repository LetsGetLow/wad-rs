use std::collections::HashMap;

pub type WaveTable = Vec<f32>;
pub type WaveTableCollection = HashMap<WaveTableType, WaveTable>;

#[derive(Debug, Clone, Copy)]
pub enum WaveTableSize {
    B256,
    B512,
    B1024,
    B2048,
    B4096,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaveTableType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

fn sine_wave(sample_size: usize) -> WaveTable {
    let mut data = WaveTable::with_capacity(sample_size);
    for i in 0..sample_size {
        let sample = (2.0 * std::f32::consts::PI * i as f32 / sample_size as f32).sin();
        data.push(sample);
    }
    data
}

fn square_wave(sample_size: usize) -> WaveTable {
    let mut data = WaveTable::with_capacity(sample_size);
    for n in 0..sample_size {
        let sample = if n < sample_size / 2 { 1.0 } else { -1.0 };
        data.push(sample);
    }
    data
}

fn sawtooth_wave(sample_size: usize) -> WaveTable {
    let mut data = WaveTable::with_capacity(sample_size);
    let denom = (sample_size - 1) as f32;
    for n in 0..sample_size {
        let sample = 2.0 * (n as f32 / denom) - 1.0;
        data.push(sample);
    }
    data
}

fn triangle_wave(sample_size: usize) -> WaveTable {
    let mut data = WaveTable::with_capacity(sample_size);
    let half = sample_size / 2;
    for n in 0..half {
        data.push(-1.0 + 2.0 * n as f32 / half as f32); // ramp up
    }
    for n in half..sample_size {
        data.push(1.0 - 2.0 * (n - half) as f32 / half as f32); // ramp down
    }
    data
}

fn noise_wave(sample_size: usize) -> WaveTable {
    let mut data = WaveTable::with_capacity(sample_size);
    for _ in 0..sample_size {
        let sample: f32 = fastrand::f32() * 2.0 - 1.0;
        data.push(sample);
    }
    data
}

pub fn generate_wave_tables(sample_size: WaveTableSize) -> WaveTableCollection {
    let size = match sample_size {
        WaveTableSize::B256 => 256,
        WaveTableSize::B512 => 512,
        WaveTableSize::B1024 => 1024,
        WaveTableSize::B2048 => 2048,
        WaveTableSize::B4096 => 4096,
    };

    let mut map = WaveTableCollection::with_capacity(5);
    map.insert(WaveTableType::Sine, sine_wave(size));
    map.insert(WaveTableType::Square, square_wave(size));
    map.insert(WaveTableType::Sawtooth, sawtooth_wave(size));
    map.insert(WaveTableType::Triangle, triangle_wave(size));
    map.insert(WaveTableType::Noise, noise_wave(size));
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generat_wave_table_collection_use_correct_sizes() {
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        for (name, data) in wave_table.iter() {
            assert_eq!(data.len(), 256, "Waveform '{:?}' has incorrect size", name);
        }

        let wave_table = generate_wave_tables(WaveTableSize::B512);
        for (name, data) in wave_table.iter() {
            assert_eq!(data.len(), 512, "Waveform '{:?}' has incorrect size", name);
        }

        let wave_table = generate_wave_tables(WaveTableSize::B1024);
        for (name, data) in wave_table.iter() {
            assert_eq!(data.len(), 1024, "Waveform '{:?}' has incorrect size", name);
        }

        let wave_table = generate_wave_tables(WaveTableSize::B2048);
        for (name, data) in wave_table.iter() {
            assert_eq!(data.len(), 2048, "Waveform '{:?}' has incorrect size", name);
        }

        let wave_table = generate_wave_tables(WaveTableSize::B4096);
        for (name, data) in wave_table.iter() {
            assert_eq!(data.len(), 4096, "Waveform '{:?}' has incorrect size", name);
        }
    }

    #[test]
    fn sine_wave_generates_correct_values() {
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        let sine_wave = wave_table.get(&WaveTableType::Sine).unwrap();
        assert!((sine_wave[0] - 0.0).abs() < 1e-6);
        assert!((sine_wave[64] - 1.0).abs() < 1e-6);
        assert!((sine_wave[128] - 0.0).abs() < 1e-6);
        assert!((sine_wave[192] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn square_wave_generates_correct_values() {
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        let square_wave = wave_table.get(&WaveTableType::Square).unwrap();
        for i in 0..128 {
            assert!((square_wave[i] - 1.0).abs() < 1e-6);
        }
        for i in 128..256 {
            assert!((square_wave[i] + 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn sawtooth_wave_generates_correct_values() {
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        let sawtooth_wave = wave_table.get(&WaveTableType::Sawtooth).unwrap();
        assert!((sawtooth_wave[0] + 1.0).abs() < 1e-6);
        assert!((sawtooth_wave[255] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn triangle_wave_generates_correct_values() {
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        let triangle_wave = wave_table.get(&WaveTableType::Triangle).unwrap();
        assert!((triangle_wave[0] + 1.0).abs() < 1e-6);
        assert!((triangle_wave[128] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn noise_wave_generates_values_in_range() {
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        let noise_wave = wave_table.get(&WaveTableType::Noise).unwrap();
        for &sample in noise_wave.iter() {
            assert!(sample <= 1.0 && sample >= -1.0);
        }
    }
}
