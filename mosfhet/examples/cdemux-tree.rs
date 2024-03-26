use mosfhet::prelude::*;
use rand::Rng;

// Apply CDEMUX gate in a tree-like unfold (in-place)
pub fn cdemux_tree(in_: &mut [Trlwe], sel: &[TrgswDft]) {
    for (i, bit) in sel.iter().enumerate().rev() {
        bit.cdemux_vectored(in_, 1 << i);
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
    let num_luts: usize = 10;

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
        if i == 0 {
            // Create a new LUT containing all values in [0, N[
            TorusPolynomial::from_fn(upper_n, |i| {
                Torus::from_unsigned(i as RawTorus, log_upper_n)
            })
        } else {
            // Create a zeroed LUT for indices other than zero
            TorusPolynomial::from_elem(upper_n, Torus::MIN)
        }
    });

    // Apply CDEMUX-tree in-place
    cdemux_tree(enc_lut.as_slice_mut(), enc_sel.as_slice());

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
