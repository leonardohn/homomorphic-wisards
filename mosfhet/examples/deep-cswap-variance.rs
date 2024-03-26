use mosfhet::prelude::*;

fn main() {
    // TFHE parameters
    let upper_n = 2048;
    let sigma = 4.884981308350689e-16f64;
    let bg_bit = 23;
    let l = 1;
    let k = 1;

    // Define number of LUTs and output precision
    let num_luts = 2048usize;

    // Generate new keys
    let trlwe_key = TrlweKey::new(upper_n, k, sigma);
    let trgsw_key = TrgswKey::new(&trlwe_key, l, bg_bit);

    // Calculate log2(num_luts)
    let log_luts = num_luts.next_power_of_two().ilog2() as usize;

    // Instantiate zeroed LUTs
    let zpoly = TorusPolynomial::from_elem(upper_n, Torus::MIN);
    let mut enc_lut = TrlweArray::from_elem(num_luts, &trlwe_key, zpoly);
    let num_values = num_luts * upper_n as usize;

    // Instantiate an array of selectors with value 1
    let torus_one = Torus::from_raw(1);
    let enc_sel = TrgswDftArray::from_elem(log_luts, &trgsw_key, torus_one, 0);

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

    // Perform deep CSWAP over the LUTs
    for (i, sel) in enc_sel.iter().enumerate() {
        // Perform a single step
        for j in 0..(1 << i) {
            sel.cswap_vectored(enc_lut.as_slice_mut(), j, 1 << i);
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
        println!(
            "Error Variance (step {i} - {num_values} values): {variance:e}"
        );
    }
}
