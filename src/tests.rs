#![cfg(test)]

use realfft::num_complex::Complex;
use realfft::RealFftPlanner;

use instr::hw::SAMPLE_RATE;
use instr::synth::Synth;

fn amplitude(harmonic: Complex<f64>) -> f64 {
    let re = harmonic.re;
    let im = harmonic.im;

    (re * re + im * im).sqrt()
}

fn fft_analyze(synth: &mut Synth) -> usize {
    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward((SAMPLE_RATE * 2) as usize);

    let mut spectrum = fft.make_output_vec();

    let mut buffer: Vec<f64> = synth
        .into_iter()
        .take((SAMPLE_RATE * 2) as usize)
        .map(|sample| sample as f64)
        .collect();

    fft.process(&mut buffer, &mut spectrum).unwrap();

    let amplitudes: Vec<f64> = spectrum
        .into_iter()
        // .skip(1)
        .map(amplitude)
        .collect();

    let peak = amplitudes.iter().copied().reduce(f64::max).unwrap();

    amplitudes
        .into_iter()
        .position(|amplitude| amplitude == peak)
        .unwrap()
}

#[test]
fn test_pitch_of_reference_note() {
    let mut synth = Synth::new();
    synth.play(60, 127);

    let fundamental = fft_analyze(&mut synth);

    assert_eq!(fundamental, 264);
}
