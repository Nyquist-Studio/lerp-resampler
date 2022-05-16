use anyhow::Result;
use nyan_resampler::{MonoFrame, OutputAudio, Resampler, SourceAudio};

const I24_MAX: i32 = 2_i32.pow(23) - 1;
const BUFFER_SIZE: usize = 256;

const INPUT_SAMPLE_RATE: f64 = 44100.0;
const OUTPUT_SAMPLE_RATE: f64 = 22050.0;

fn main() -> Result<()> {
    let samples = {
        let mut reader = hound::WavReader::open("sine-660Hz-mono.wav")?;
        reader
            .samples::<i32>()
            .map(|result| result.unwrap_or(0))
            .map(|sample| sample as f32 / I24_MAX as f32)
            .map(|value| MonoFrame { value })
            .collect::<Vec<MonoFrame>>()
    };

    let source_length_in_samples = samples.len() as f64;
    let mut source = SourceAudio {
        sample_rate: INPUT_SAMPLE_RATE,
        audio: samples,
        playhead_position_in_fractional_samples: 0.0,
    };

    let mut output_combined = vec![];
    let mut output_buffer = vec![MonoFrame::default(); BUFFER_SIZE];

    let mut resampler = Resampler::new(BUFFER_SIZE);
    while source.playhead_position_in_fractional_samples < source_length_in_samples {
        let output = OutputAudio {
            sample_rate: OUTPUT_SAMPLE_RATE,
            buffer: &mut output_buffer,
        };
        resampler.resample(&mut source, output);
        output_combined.extend_from_slice(&output_buffer);
    }

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: OUTPUT_SAMPLE_RATE as u32,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("resampled.wav", spec)?;
    for output in output_combined {
        let amplitude = (I24_MAX as f32 * output.value) as i32;
        writer.write_sample(amplitude)?;
        writer.write_sample(amplitude)?;
    }
    writer.finalize()?;

    Ok(())
}
