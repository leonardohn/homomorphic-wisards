use std::fmt;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{black_box, criterion_group, criterion_main};
use mosfhet::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParamSet {
    pub sigma: f64,
    pub upper_n: u32,
    pub l: u32,
    pub bg_bit: u32,
}

impl ParamSet {
    pub fn new(sigma: f64, upper_n: u32, l: u32, bg_bit: u32) -> Self {
        Self {
            sigma,
            upper_n,
            l,
            bg_bit,
        }
    }
}

impl fmt::Display for ParamSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[rustfmt::skip] let ParamSet { sigma, upper_n, l, bg_bit } = *self;
        write!(f, "sigma={sigma:e}/upper_n={upper_n}/l={l}/Bg_bit={bg_bit}")
    }
}

#[rustfmt::skip]
static SIGMA_UPPER_N_VALUES: [(f64, u32); 3] = [
    (5.6152343750000000e-04, 512), // 1.15 * (2 ** -11)
    (5.5134296417236328e-08, 1024), // 1.85 * (2 ** -25)
    (4.8849813083506888e-16, 2048), // 1.10 * (2 ** -51)
];

fn trlwe_trgsw_mul(crit: &mut Criterion) {
    // Create a new benchmark group
    let mut group = crit.benchmark_group("trlwe_trgsw_mul");

    // Iterate through sigma and upper_n parameters
    for (sigma, upper_n) in SIGMA_UPPER_N_VALUES {
        // Iterate through l and bg_bit parameters
        for (l, bg_bit) in (1..55).map(|l| (l, 54 / l)) {
            // Create a parameter set from the raw parameters
            #[rustfmt::skip]
            let set = ParamSet { sigma, upper_n, l, bg_bit };

            // Create unique identifier for this benchmark
            let bench_id = BenchmarkId::from_parameter(set);

            // Generate new keys
            let trlwe_key = TrlweKey::new(upper_n, 1, sigma);
            let trgsw_key = TrgswKey::new(&trlwe_key, l, bg_bit);

            // Generate and encrypt a zeroed Trlwe
            let poly = TorusPolynomial::from_elem(upper_n, Torus::MIN);
            let trlwe = Trlwe::new(poly, &trlwe_key);

            // Generate and encrypt a TrgswDft
            let sel = Torus::from_raw(black_box(1));
            let trgsw = Trgsw::new(sel, 0, &trgsw_key);
            let trgsw_dft = TrgswDft::from_trgsw(&trgsw);

            // Benchmark the target function
            group.bench_function(bench_id, |b| {
                b.iter(|| TrlweDft::mul_trlwe_dft(&trlwe, &trgsw_dft));
            });
        }
    }

    // Finish the benchmark group
    group.finish();
}

criterion_group!(benches, trlwe_trgsw_mul);
criterion_main!(benches);
