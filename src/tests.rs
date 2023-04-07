#![cfg(test)]

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

use instr::hw::SAMPLE_RATE;
use instr::synth::Synth;

fn amplitude(harmonic: Complex<f64>) -> f64 {
    let re = harmonic.re;
    let im = harmonic.im;

    (re * re + im * im).sqrt()
}

fn determine_n_strongest_frequencies(synth: &mut Synth, n: usize, resolution: f64) -> Vec<f64> {
    fn tuple_swap<T, U>((a, b): (T, U)) -> (U, T) {
        (b, a)
    }

    let fft_size = (SAMPLE_RATE * 2 * (resolution.recip() as u32)) as usize;

    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut spectrum: Vec<Complex<f64>> = synth
        .into_iter()
        .take(fft_size)
        .map(|sample| Complex {
            re: sample as f64,
            im: 0.0,
        })
        .collect();

    fft.process(&mut spectrum);

    let mut amplitudes: Vec<(f64, usize)> = spectrum
        .into_iter()
        // .skip(1)
        .map(amplitude)
        .enumerate()
        .filter(|(frequency, _)| (*frequency as u32) < (SAMPLE_RATE / 2))
        .map(tuple_swap)
        .collect();

    amplitudes
        .sort_by(|(amplitude1, _), (amplitude2, _)| amplitude1.partial_cmp(amplitude2).unwrap());
    amplitudes.reverse();

    amplitudes
        .into_iter()
        .take(n)
        .map(|(_, frequency)| frequency as f64 * resolution)
        .collect()
}

#[test]
fn test_pitch_of_reference_note() {
    let mut synth = Synth::new();
    synth.play(60, 127);

    let fundamental = determine_n_strongest_frequencies(&mut synth, 1, 1.0)[0];

    assert_eq!(fundamental, 264.0);
}

#[test]
fn test_3_2_from_264hz() {
    let mut synth = Synth::new();
    synth.play(60, 127);
    synth.play(67, 127);

    let fundamentals = determine_n_strongest_frequencies(&mut synth, 2, 1.0);

    assert_eq!(fundamentals[0], 264.0);
    assert_eq!(fundamentals[1], 396.0);
}

#[test]
fn test_9_8_from_396hz() {
    let mut synth = Synth::new();
    synth.play(67, 127);
    synth.play(69, 127);

    let fundamentals = determine_n_strongest_frequencies(&mut synth, 2, 0.5);

    assert_eq!(fundamentals[0], 396.0);
    assert_eq!(fundamentals[1], 445.5);
}
