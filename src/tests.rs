#![cfg(test)]

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

use instr::file::render_to_file;
use instr::hw::SAMPLE_RATE;
use instr::synth::Synth;

const TOLERANCE: f64 = 0.085;

fn amplitude(harmonic: Complex<f64>) -> f64 {
    let re = harmonic.re;
    let im = harmonic.im;

    (re * re + im * im).sqrt()
}

fn cent_difference(f1: f64, f2: f64) -> f64 {
    (f64::log2(f1 / f2) * 1200.0).abs()
}

fn determine_n_strongest_frequencies(synth: &mut Synth, n: usize) -> Vec<f64> {
    fn tuple_swap<T, U>((a, b): (T, U)) -> (U, T) {
        (b, a)
    }

    let resolution = 0.5_f64.powi(5);

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
        .filter_map(|harmonic| {
            let a = amplitude(harmonic);

            if a > 0.0 {
                Some(a)
            } else {
                None
            }
        })
        .enumerate()
        .filter(|(frequency, _)| (*frequency as u32) < (SAMPLE_RATE / 2))
        .map(tuple_swap)
        .collect();

    amplitudes
        .sort_by(|(amplitude1, _), (amplitude2, _)| amplitude1.partial_cmp(amplitude2).unwrap());
    amplitudes.reverse();

    let mut amplitudes: Vec<f64> = amplitudes
        .into_iter()
        .take(n)
        .map(|(_, frequency)| frequency as f64 * resolution)
        .collect();

    amplitudes.sort_by(|f1, f2| f1.partial_cmp(f2).unwrap());

    amplitudes
}

fn test_inexact_n(
    (midi_note, freq): (u8, f64),
    intervals: &[(u8, f64, f64)],
    factors: &[u8],
    tolerance: f64,
) {
    let mut synth = Synth::new();

    for &factor in factors {
        synth.change_tuning(factor);
    }

    synth.play(midi_note, 127);

    for &(midi_interval, _, _) in intervals {
        synth.play(midi_note + midi_interval, 127);
    }

    let n = intervals.len() + 1;
    let fundamentals = determine_n_strongest_frequencies(&mut synth, n);

    assert_eq!(fundamentals[0], freq);
    for (i, &(_, num, denom)) in intervals.into_iter().enumerate() {
        assert!(
            cent_difference(fundamentals[i + 1], freq * (num / denom)) < tolerance,
            "{}",
            cent_difference(fundamentals[i + 1], freq * (num / denom))
        );
    }
}

fn test_inexact_1(fundamental: (u8, f64), tolerance: f64) {
    test_inexact_n(fundamental, &[], &[], tolerance);
}

fn test_inexact_2(
    fundamental: (u8, f64),
    interval: (u8, f64, f64),
    factors: &[u8],
    tolerance: f64,
) {
    test_inexact_n(fundamental, &[interval], factors, tolerance);
}

mod basic {
    use super::*;

    #[test]
    fn test_no_notes() {
        let mut synth = Synth::new();

        let fundamentals = determine_n_strongest_frequencies(&mut synth, 1);

        assert_eq!(fundamentals.len(), 0);
    }

    #[test]
    fn test_c_264() {
        test_inexact_1((60, 264.0), TOLERANCE);
    }
}

mod volume {
    use super::*;

    #[test]
    fn test_silent() {
        let mut synth = Synth::new();
        synth.play(60, 0);

        let fundamentals = determine_n_strongest_frequencies(&mut synth, 1);

        assert_eq!(fundamentals.len(), 0);
    }

    #[test]
    fn test_almost_silent() {
        let mut synth = Synth::new();
        synth.play(60, 1);

        let fundamentals = determine_n_strongest_frequencies(&mut synth, 1);

        assert_eq!(fundamentals[0], 264.0);
    }

    #[test]
    fn test_envelope() {
        let mut synth = Synth::new();
        synth.play(60, 127);

        let a = &mut synth;

        let mut first: Vec<i16> = a.into_iter().take(SAMPLE_RATE as usize * 2).collect();

        synth.silence(60);

        let b = &mut synth;

        let mut second: Vec<i16> = b.into_iter().take(SAMPLE_RATE as usize * 2).collect();

        first.append(&mut second);

        render_to_file("envelope_test.wav", first);
    }

    #[test]
    fn test_envelope_change_volume() {
        let mut synth = Synth::new();
        synth.play(60, 127);

        let a = &mut synth;

        let mut first: Vec<i16> = a.into_iter().take(SAMPLE_RATE as usize * 2).collect();

        synth.play(60, 63);

        let b = &mut synth;

        let mut second: Vec<i16> = b.into_iter().take(SAMPLE_RATE as usize * 2).collect();

        first.append(&mut second);
        render_to_file("envelope_test_change_volume.wav", first);
    }
}

mod fundamental {
    use super::*;

    #[test]
    fn test_change_fundamental_identity() {
        let mut synth = Synth::new();
        synth.play(60, 127);

        synth.change_fundamental(60 - 12);

        let fundamentals = determine_n_strongest_frequencies(&mut synth, 1);

        assert_eq!(fundamentals[0], 264.0);
    }

    // #[test]
    // fn test_change_fundamental_pythagorean() {
    //     let mut synth = Synth::new();
    //     synth.play(60, 127);
    //
    //     synth.change_fundamental(66 - 12);
    //
    //     let fundamentals = determine_n_strongest_frequencies(&mut synth, 1);
    //
    //     assert_eq!(fundamentals[0], 264.0);
    // }
}

mod dyads {
    use super::*;

    #[test]
    fn test_33_32() {
        test_inexact_2((60, 264.0), (1, 33.0, 32.0), &[65], TOLERANCE);
    }

    #[test]
    fn test_125_121() {
        test_inexact_2((60, 264.0), (1, 125.0, 121.0), &[64, 65, 67], TOLERANCE);
    }

    #[test]
    fn test_28_27() {
        test_inexact_2((60, 264.0), (1, 28.0, 27.0), &[69], TOLERANCE);
    }

    #[test]
    fn test_80_77() {
        test_inexact_2((60, 264.0), (1, 80.0, 77.0), &[64, 65, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_256_245() {
        test_inexact_2((60, 264.0), (1, 256.0, 245.0), &[64, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_256_243() {
        test_inexact_2((60, 264.0), (1, 256.0, 243.0), &[], TOLERANCE);
    }

    #[test]
    fn test_128_121() {
        test_inexact_2((60, 264.0), (1, 128.0, 121.0), &[65, 67], TOLERANCE);
    }

    // Depends on 5:4
    #[test]
    fn test_16_15() {
        test_inexact_2((60, 264.0), (1, 16.0, 15.0), &[64], TOLERANCE);
    }

    #[test]
    fn test_121_112() {
        test_inexact_2((60, 264.0), (1, 121.0, 112.0), &[65, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_11_10() {
        test_inexact_2((60, 264.0), (2, 11.0, 10.0), &[64, 65, 67], TOLERANCE);
    }

    #[test]
    fn test_9_8() {
        test_inexact_2((60, 264.0), (2, 9.0, 8.0), &[], TOLERANCE);
    }

    #[test]
    fn test_8_7() {
        test_inexact_2((60, 264.0), (2, 8.0, 7.0), &[67, 69], TOLERANCE);
    }

    #[test]
    fn test_64_55() {
        test_inexact_2((60, 264.0), (3, 64.0, 55.0), &[64, 65, 67], TOLERANCE);
    }

    #[test]
    fn test_400_343() {
        test_inexact_2((60, 264.0), (3, 400.0, 343.0), &[64, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_7_6() {
        test_inexact_2((60, 264.0), (3, 7.0, 6.0), &[69], TOLERANCE);
    }

    #[test]
    fn test_32_27() {
        test_inexact_2((60, 264.0), (3, 32.0, 27.0), &[], TOLERANCE);
    }

    #[test]
    fn test_6_5() {
        test_inexact_2((60, 264.0), (3, 6.0, 5.0), &[64], TOLERANCE);
    }

    #[test]
    fn test_77_64() {
        test_inexact_2((60, 264.0), (3, 77.0, 64.0), &[65, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_625_512() {
        test_inexact_2((60, 264.0), (3, 625.0, 512.0), &[64, 67], TOLERANCE);
    }

    #[test]
    fn test_11_9() {
        test_inexact_2((60, 264.0), (3, 11.0, 9.0), &[65], TOLERANCE);
    }

    #[test]
    fn test_27_22() {
        test_inexact_2((60, 264.0), (4, 27.0, 22.0), &[65], TOLERANCE);
    }

    #[test]
    fn test_5_4() {
        test_inexact_2((60, 264.0), (4, 5.0, 4.0), &[64, 67], TOLERANCE);
    }

    #[test]
    fn test_81_64() {
        test_inexact_2((60, 264.0), (4, 81.0, 64.0), &[], TOLERANCE);
    }

    #[test]
    fn test_14_11() {
        test_inexact_2((60, 264.0), (4, 14.0, 11.0), &[65, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_9_7() {
        test_inexact_2((60, 264.0), (4, 9.0, 7.0), &[65, 69], TOLERANCE);
    }

    #[test]
    fn test_1331_1024() {
        test_inexact_2((60, 264.0), (5, 1331.0, 1024.0), &[65, 67], TOLERANCE);
    }

    #[test]
    fn test_64_49() {
        test_inexact_2((60, 264.0), (5, 64.0, 49.0), &[67, 69], TOLERANCE);
    }

    #[test]
    fn test_160_121() {
        test_inexact_2((60, 264.0), (5, 160.0, 121.0), &[64, 65, 67], TOLERANCE);
    }

    #[test]
    fn test_4_3() {
        test_inexact_2((60, 264.0), (5, 4.0, 3.0), &[], TOLERANCE);
    }

    #[test]
    fn test_11_8() {
        test_inexact_2((60, 264.0), (6, 11.0, 8.0), &[65, 67], TOLERANCE);
    }

    #[test]
    fn test_25_18() {
        test_inexact_2((60, 264.0), (6, 25.0, 18.0), &[64], TOLERANCE);
    }

    #[test]
    fn test_7_5() {
        test_inexact_2((60, 264.0), (6, 7.0, 5.0), &[64, 69], TOLERANCE);
    }

    // This test leads to OOM condition, unfortunately
    // #[test]
    // fn test_729_512() {
    //     test_inexact_2((60, 264.0), 6, &[64, 67], 0.000125, 375.890625);
    // }

    #[test]
    fn test_729_512() {
        test_inexact_2((60, 264.0), (6, 729.0, 512.0), &[], TOLERANCE);
    }

    #[test]
    fn test_81_56() {
        test_inexact_2((60, 264.0), (6, 81.0, 56.0), &[69], TOLERANCE);
    }

    #[test]
    fn test_3_2() {
        test_inexact_2((60, 264.0), (7, 3.0, 2.0), &[], TOLERANCE);
    }

    #[test]
    fn test_121_80() {
        test_inexact_2((60, 264.0), (7, 121.0, 80.0), &[64, 65, 67], TOLERANCE);
    }

    #[test]
    fn test_49_32() {
        test_inexact_2((60, 264.0), (7, 49.0, 32.0), &[67, 69], TOLERANCE);
    }

    #[test]
    fn test_2048_1331() {
        test_inexact_2((60, 264.0), (7, 2048.0, 1331.0), &[65, 67], TOLERANCE);
    }

    #[test]
    fn test_14_9() {
        test_inexact_2((60, 264.0), (8, 14.0, 9.0), &[69], TOLERANCE);
    }

    #[test]
    fn test_11_7() {
        test_inexact_2((60, 264.0), (8, 11.0, 7.0), &[65, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_128_81() {
        test_inexact_2((60, 264.0), (8, 128.0, 81.0), &[], TOLERANCE);
    }

    #[test]
    fn test_8_5() {
        test_inexact_2((60, 264.0), (8, 8.0, 5.0), &[64], TOLERANCE);
    }

    #[test]
    fn test_44_27() {
        test_inexact_2((60, 264.0), (8, 44.0, 27.0), &[65], TOLERANCE);
    }

    #[test]
    fn test_18_11() {
        test_inexact_2((60, 264.0), (9, 18.0, 11.0), &[65], TOLERANCE);
    }

    #[test]
    fn test_1024_625() {
        test_inexact_2((60, 264.0), (9, 1024.0, 625.0), &[64, 67], TOLERANCE);
    }

    #[test]
    fn test_128_77() {
        test_inexact_2((60, 264.0), (9, 128.0, 77.0), &[65, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_5_3() {
        test_inexact_2((60, 264.0), (9, 5.0, 3.0), &[64], TOLERANCE);
    }

    #[test]
    fn test_27_16() {
        test_inexact_2((60, 264.0), (9, 27.0, 16.0), &[], TOLERANCE);
    }

    #[test]
    fn test_12_7() {
        test_inexact_2((60, 264.0), (9, 12.0, 7.0), &[69], TOLERANCE);
    }

    #[test]
    fn test_343_200() {
        test_inexact_2((60, 264.0), (9, 343.0, 200.0), &[64, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_55_32() {
        test_inexact_2((60, 264.0), (9, 55.0, 32.0), &[64, 65, 67], TOLERANCE);
    }

    #[test]
    fn test_7_4() {
        test_inexact_2((60, 264.0), (10, 7.0, 4.0), &[67, 69], TOLERANCE);
    }

    #[test]
    fn test_16_9() {
        test_inexact_2((60, 264.0), (10, 16.0, 9.0), &[], TOLERANCE);
    }

    #[test]
    fn test_9_5() {
        test_inexact_2((60, 264.0), (10, 9.0, 5.0), &[64], TOLERANCE);
    }

    #[test]
    fn test_20_11() {
        test_inexact_2((60, 264.0), (10, 20.0, 11.0), &[64, 65, 67], TOLERANCE);
    }

    #[test]
    fn test_11_6() {
        test_inexact_2((60, 264.0), (10, 11.0, 6.0), &[65], TOLERANCE);
    }

    #[test]
    fn test_15_8() {
        test_inexact_2((60, 264.0), (11, 15.0, 8.0), &[64], TOLERANCE);
    }

    #[test]
    fn test_121_64() {
        test_inexact_2((60, 264.0), (11, 121.0, 64.0), &[65, 67], TOLERANCE);
    }

    #[test]
    fn test_243_128() {
        test_inexact_2((60, 264.0), (11, 243.0, 128.0), &[], TOLERANCE);
    }

    #[test]
    fn test_21_11() {
        test_inexact_2((60, 264.0), (11, 21.0, 11.0), &[65, 69], TOLERANCE);
    }

    #[test]
    fn test_245_128() {
        test_inexact_2((60, 264.0), (11, 245.0, 128.0), &[64, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_77_40() {
        test_inexact_2((60, 264.0), (11, 77.0, 40.0), &[64, 65, 67, 69], TOLERANCE);
    }

    #[test]
    fn test_27_14() {
        test_inexact_2((60, 264.0), (11, 27.0, 14.0), &[69], TOLERANCE);
    }

    #[test]
    fn test_64_33() {
        test_inexact_2((60, 264.0), (11, 64.0, 33.0), &[65], TOLERANCE);
    }

    #[test]
    fn test_empty_interval() {
        let mut synth = Synth::new();

        synth.change_tuning(67);
        synth.play(61, 127);

        let fundamentals = determine_n_strongest_frequencies(&mut synth, 1);

        assert_eq!(fundamentals.len(), 0);
    }
}
