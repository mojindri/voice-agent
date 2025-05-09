pub struct AudioProcessor {
    sample_rate: u32,
    channels: u16,
}

impl AudioProcessor {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }

    pub fn normalize(&self, audio_data: &mut [f32]) {
        let max_amplitude = audio_data.iter().map(|&x| x.abs()).fold(0.0, f32::max);

        if max_amplitude > 0.0 {
            let scale = 0.8 / max_amplitude;
            for sample in audio_data.iter_mut() {
                *sample *= scale;
            }
        }
    }

    pub fn remove_silence(&self, audio_data: &[f32], threshold: f32) -> Vec<f32> {
        audio_data
            .iter()
            .filter(|&&x| x.abs() > threshold)
            .copied()
            .collect()
    }

    pub fn apply_noise_reduction(&self, audio_data: &mut [f32]) {
        let noise_floor = -50.0;
        let threshold = 10.0f32.powf(noise_floor / 20.0);

        for sample in audio_data.iter_mut() {
            if sample.abs() < threshold {
                *sample = 0.0;
            }
        }
    }

    pub fn apply_bandpass_filter(&self, audio_data: &mut [f32]) {
        let low_freq = 80.0;
        let high_freq = 6000.0;

        let dt = 1.0 / self.sample_rate as f32;
        let alpha_low = 2.0 * std::f32::consts::PI * low_freq * dt;
        let alpha_high = 2.0 * std::f32::consts::PI * high_freq * dt;

        let mut prev_sample = 0.0f32;
        let mut prev_prev_sample = 0.0f32;

        for sample in audio_data.iter_mut() {
            let current = *sample;
            let high_pass = current - 2.0 * prev_sample + prev_prev_sample;
            let low_pass = alpha_high * high_pass + (1.0 - alpha_high) * prev_sample;

            *sample = low_pass;
            prev_prev_sample = prev_sample;
            prev_sample = low_pass;
        }
    }

    pub fn process_audio(&self, audio_data: &mut [f32]) {
        self.apply_noise_reduction(audio_data);
        self.apply_bandpass_filter(audio_data);
        self.normalize(audio_data);
    }

    pub fn resample(&self, audio_data: &[f32], target_sample_rate: u32) -> Vec<f32> {
        if self.sample_rate == target_sample_rate {
            return audio_data.to_vec();
        }

        let ratio = target_sample_rate as f32 / self.sample_rate as f32;
        let new_length = (audio_data.len() as f32 * ratio) as usize;
        let mut result = Vec::with_capacity(new_length);

        for i in 0..new_length {
            let pos = i as f32 / ratio;
            let index = pos as usize;
            let fraction = pos - index as f32;

            if index + 1 < audio_data.len() {
                let sample =
                    audio_data[index] * (1.0 - fraction) + audio_data[index + 1] * fraction;
                result.push(sample);
            } else {
                result.push(audio_data[index]);
            }
        }

        result
    }
}
