#[derive(Default, Copy, Clone)]
pub struct MonoFrame {
    pub value: f32,
}

pub struct SourceAudio {
    pub sample_rate: f64,
    pub audio: Vec<MonoFrame>,
    pub playhead_position_in_fractional_samples: f64,
}

pub struct OutputAudio<'a> {
    pub sample_rate: f64,
    pub buffer: &'a mut [MonoFrame],
}

pub struct Resampler {
    source_buffer_positions_in_fractional_samples: Vec<f64>,
}

impl<'a> Resampler {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            source_buffer_positions_in_fractional_samples: vec![0.0; buffer_size],
        }
    }

    pub fn resample(&mut self, source: &mut SourceAudio, output: OutputAudio<'a>) {
        // Calculate source sample positions
        let source_samples_per_output_sample = source.sample_rate / output.sample_rate;
        for i in 0..output.buffer.len() {
            self.source_buffer_positions_in_fractional_samples[i] = source
                .playhead_position_in_fractional_samples
                + i as f64 * source_samples_per_output_sample;
        }

        // Interpolate each source sample
        for (output_buffer_index, source_buffer_position_in_fractional_samples) in self
            .source_buffer_positions_in_fractional_samples
            .iter()
            .enumerate()
        {
            let source_position_before = source_buffer_position_in_fractional_samples.floor();
            let source_frame_before = source
                .audio
                .get(source_position_before as usize)
                .copied()
                .unwrap_or_default();

            let source_position_after = source_buffer_position_in_fractional_samples.ceil();
            let source_frame_after = source
                .audio
                .get(source_position_after as usize)
                .copied()
                .unwrap_or_default();

            if source_position_before == source_position_after {
                output.buffer[output_buffer_index].value = source_frame_before.value;
            } else {
                output.buffer[output_buffer_index].value = lerp(
                    Point {
                        x: source_position_before,
                        y: source_frame_before.value as f64,
                    },
                    Point {
                        x: source_position_after,
                        y: source_frame_after.value as f64,
                    },
                    *source_buffer_position_in_fractional_samples,
                ) as f32;
            }
        }

        // Update playhead position
        source.playhead_position_in_fractional_samples = self
            .source_buffer_positions_in_fractional_samples[output.buffer.len() - 1]
            + source_samples_per_output_sample;
    }
}

struct Point {
    pub x: f64,
    pub y: f64,
}

fn lerp(p0: Point, p1: Point, x: f64) -> f64 {
    debug_assert_ne!(p1.x - p0.x, 0.0);
    let slope = (x - p0.x) / (p1.x - p0.x);
    // See: https://ccrma.stanford.edu/~jos/pasp/One_Multiply_Linear_Interpolation.html
    p0.y + slope * (p1.y - p0.y)
}
