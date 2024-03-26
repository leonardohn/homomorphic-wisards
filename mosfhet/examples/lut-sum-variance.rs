use mosfhet::prelude::*;

fn main() {
    // TFHE parameters
    let upper_n = 2048;
    let sigma = 4.884981308350689e-16f64;
    let k = 1;

    // Define number of LUTs and sums
    let num_luts = 16;
    let num_sums = 4096;

    dbg!(num_luts);
    dbg!(num_sums);

    // Generate new keys
    let trlwe_key = TrlweKey::new(upper_n, k, sigma);

    // Instantiate zeroed noiseless LUTs
    let mut enc_lut = TrlweArray::zeroed_noiseless(num_luts, k, upper_n);
    let num_values = num_luts * upper_n as usize;

    // Generate new masks and add to the LUT
    for _ in 0..num_sums {
        // Encrypt a new mask (zeroed LUT with one on a single index)
        let enc_mask = TrlweArray::zeroed(num_luts, &trlwe_key);

        // Add each mask Trlwe to the LUT
        enc_lut
            .iter_mut()
            .zip(enc_mask.iter())
            .for_each(|(lut, mask)| lut.add_assign(mask));
    }

    // Measure the variance for all LUT values
    let sum_variances = enc_lut
        .iter()
        .map(|lut| lut.phase(&trlwe_key))
        .flat_map(|poly| poly.iter().cloned().collect::<Vec<_>>())
        .map(|val| val.distance(Torus::MIN).into_double().powi(2))
        .sum::<f64>();

    // Calculate the variance
    let variance = sum_variances / num_values as f64;

    // Display the variance
    println!("Error Variance: {variance:e}");
}
