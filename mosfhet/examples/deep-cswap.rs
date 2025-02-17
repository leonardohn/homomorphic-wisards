use mosfhet::prelude::*;
use rand::Rng;

// Apply CSWAP gate between every pair of elements (in-place)
pub fn deep_cswap(in_: &mut [Trlwe], sel: &[TrgswDft]) {
    for (i, bit) in sel.iter().enumerate() {
        for j in 0..(1 << i) {
            bit.cswap_vectored(in_, j, 1 << i);
        }
    }
}

fn main() {
    // TFHE parameters
    let upper_n = 2048u32;
    let sigma = 4.884981308350689e-16f64;
    let bg_bit = 23;
    let l = 1;
    let k = 1;

    // Define number of LUTs
    let num_luts: usize = 16;

    // Generate new keys
    let trlwe_key = TrlweKey::new(upper_n, k, sigma);
    let trgsw_key = TrgswKey::new(&trlwe_key, l, bg_bit);
    let tlwe_key = TlweKey::from_trlwe_key(&trlwe_key);

    // Calculate log2(upper_n) and ceil(log2(num_luts))
    let log_upper_n = upper_n.ilog2() as usize;
    let log_num_luts = num_luts.next_power_of_two().ilog2() as usize;

    // Generate a random selector and encrypt
    let plt_sel = rand::thread_rng().gen_range(0usize..num_luts);
    let enc_sel = TrgswDftArray::from_fn(log_num_luts, &trgsw_key, |i| {
        let bit = (plt_sel as RawTorus >> i) & 1;
        (Torus::from_raw(bit), 0)
    });

    // Encrypt the LUT
    let mut enc_lut = TrlweArray::from_fn(num_luts, &trlwe_key, |i| {
        let elem = Torus::from_unsigned(i as RawTorus, log_upper_n);
        TorusPolynomial::from_elem(upper_n, elem)
    });

    // Apply deep CSWAP over the LUTs
    deep_cswap(enc_lut.as_slice_mut(), enc_sel.as_slice());

    for k in 0..num_luts {
        // Decrypt all LUTs from output
        let plt_values: Vec<RawTorus> = (0..(upper_n as usize))
            .map(|i| {
                Tlwe::from_trlwe(&enc_lut[k], i)
                    .phase(&tlwe_key)
                    .into_unsigned(log_upper_n)
            })
            .collect();

        // Display first and last elements in the array
        let show_n = 6;
        let head_n = 0..show_n;
        let tail_n = (upper_n as usize - show_n)..(upper_n as usize);
        let marker = if plt_sel == k { "X" } else { " " };

        let head = plt_values[head_n]
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        let tail = plt_values[tail_n]
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        println!("[{marker}] LUT #{k}: [{head}, ..., {tail}]");
    }
}
