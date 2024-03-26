use mosfhet::prelude::*;

fn main() {
    // TFHE parameters
    let upper_n = 2048;
    let sigma = 4.884981308350689e-16f64;
    let bg_bit = 23;
    let l = 1;
    let k = 1;

    // Define number of LUTs
    let num_luts = 2048;

    // Generate new keys
    let trlwe_key = TrlweKey::new(upper_n, k, sigma);
    let trgsw_key = TrgswKey::new(&trlwe_key, l, bg_bit);

    // Calculate log2(upper_n)
    let log_upper_n = upper_n.ilog2() as usize;

    // Instantiate zeroed LUTs
    let mut enc_lut = TrlweArray::zeroed(num_luts, &trlwe_key);
    let num_values = num_luts * upper_n as usize;

    // Instantiate an array of selectors with value 1
    let torus_one = Torus::from_raw(1);
    let enc_sel =
        TrgswDftArray::from_elem(log_upper_n, &trgsw_key, torus_one, 0);

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
    println!("Error Variance (initial - {num_values} values): {variance:e}");

    // Perform blind rotation over each LUT and measure variance
    for (i, sel) in enc_sel.iter().enumerate() {
        // Perform a single blind rotation step over each LUT
        for lut in enc_lut.iter_mut() {
            let tmp = lut.mul_by_xai_pred(upper_n * 2 - (1 << i));
            let tmp_dft = TrlweDft::mul_trlwe_dft(&tmp, sel);
            lut.add_assign(&Trlwe::from_dft(&tmp_dft));
        }

        // Measure the variances for all LUT values
        let sum_variances = enc_lut
            .iter()
            .map(|lut| lut.phase(&trlwe_key))
            .flat_map(|poly| poly.iter().cloned().collect::<Vec<_>>())
            .map(|val| val.distance(Torus::MIN).into_double().powi(2))
            .sum::<f64>();

        // Calculate the variance
        let variance = sum_variances / num_values as f64;

        // Display the variance
        println!(
            "Error Variance (step {i} - {num_values} values): {variance:e}"
        );
    }
}
