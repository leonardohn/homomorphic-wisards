use std::collections::HashMap;

use mosfhet::prelude::*;
use rand::Rng;

fn main() {
    // TFHE parameters
    let upper_n = 2048u32;
    let sigma = 4.884981308350689e-16f64;
    let k = 1;

    // Define number of LUTs and output precision
    let num_luts = 1usize;
    let num_sums = 62000usize;
    let out_prec = 16;

    dbg!(num_luts);
    dbg!(num_sums);
    dbg!(out_prec);

    // Generate new keys
    let trlwe_key = TrlweKey::new(upper_n, k, sigma);
    let tlwe_key = TlweKey::from_trlwe_key(&trlwe_key);

    // Instantiate zeroed noiseless LUTs
    let zpoly = TorusPolynomial::from_elem(upper_n, Torus::MIN);
    let mut enc_lut =
        TrlweArray::from_elem_noiseless(num_luts, k, upper_n, zpoly);

    // Pick a random position between the LUTs
    let up_index = rand::thread_rng().gen_range(0..num_luts);
    let lo_index = rand::thread_rng().gen_range(0..(upper_n as usize));

    dbg!(up_index);
    dbg!(lo_index);

    // Generate new masks and add to the LUT
    for _ in 0..num_sums {
        // Encrypt a new mask (zeroed LUT with one on a single index)
        let enc_mask = TrlweArray::from_fn(num_luts, &trlwe_key, |i| {
            TorusPolynomial::from_fn(upper_n, |j| {
                let cond = i == up_index && j == lo_index;
                Torus::from_unsigned(cond as RawTorus, out_prec)
            })
        });

        // Add each mask Trlwe to the LUT
        enc_lut
            .as_slice_mut()
            .iter_mut()
            .zip(enc_mask.as_slice().iter())
            .for_each(|(lut, mask)| lut.add_assign(mask));
    }

    // Decrypt and display an histogram of LUT values
    for t in 0..num_luts {
        // Calculate the histogram from decrypted values
        let histogram = (0..(upper_n as usize))
            .map(|i| {
                // Decrypt the value
                Tlwe::from_trlwe(&enc_lut[t], i)
                    .phase(&tlwe_key)
                    .into_unsigned(out_prec)
            })
            .fold(HashMap::new(), |mut acc, v| {
                // Increment the counter for the value
                if let Some(count) = acc.insert(v, 1usize) {
                    acc.insert(v, count + 1);
                }
                acc
            });

        // Display the histogram
        println!("LUT #{t}: {histogram:?}");
    }
}
