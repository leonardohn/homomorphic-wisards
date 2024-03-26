mod util;

use std::cell::RefCell;
use std::collections::HashSet;
use std::num::{NonZeroU16, NonZeroU8, NonZeroUsize};
use std::path::PathBuf;
use std::thread::available_parallelism;
use std::time::Instant;

#[cfg(feature = "time-tracking")]
use std::sync::atomic::AtomicU64;

#[cfg(any(feature = "time-tracking", feature = "noise-tracking"))]
use std::sync::atomic::Ordering;

#[cfg(feature = "noise-tracking")]
use atomic_float::AtomicF64;

use indicatif::ParallelProgressIterator;
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use mosfhet::prelude::*;
use rayon::prelude::*;
use thread_local::ThreadLocal;
use wisard::dataset::Dataset;
use wisard::encode::{LinearThermometer, LogThermometer};
use wisard::encode::{Permute, SampleEncoder, Slice};

use crate::util::*;

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Opt {
    #[arg(long)]
    num_labels: Option<NonZeroU8>,

    #[arg(long)]
    #[arg(default_value_t = NonZeroU8::new(8).unwrap())]
    data_bits: NonZeroU8,

    #[arg(long)]
    data_limit: Option<NonZeroU8>,

    #[arg(long)]
    #[arg(default_value_t = 0)]
    data_skip: u8,

    #[arg(long)]
    train_data: PathBuf,

    #[arg(long)]
    #[arg(default_value_t = u32::MAX)]
    train_limit: u32,

    #[arg(long)]
    #[arg(default_value_t = u32::MIN)]
    train_skip: u32,

    #[arg(long)]
    test_data: PathBuf,

    #[arg(long)]
    #[arg(default_value_t = u32::MAX)]
    test_limit: u32,

    #[arg(long)]
    #[arg(default_value_t = u32::MIN)]
    test_skip: u32,

    #[arg(long)]
    address_size: NonZeroU8,

    #[arg(long)]
    counter_size: NonZeroU8,

    #[arg(long)]
    therm_size: NonZeroU8,

    #[arg(long, value_enum)]
    #[arg(default_value_t = ThermType::Linear)]
    therm_type: ThermType,

    #[arg(long, value_enum)]
    #[arg(default_value_t = ActivationType::Binary)]
    activation: ActivationType,

    #[arg(long)]
    #[arg(default_value_t = 0)]
    threshold: RawTorus,

    #[arg(long)]
    #[arg(default_value_t = rand::random())]
    seed: u128,

    #[arg(long)]
    sigma: f64,

    #[arg(long)]
    l: NonZeroU8,

    #[arg(long)]
    bg_bit: NonZeroU8,

    #[arg(long)]
    upper_n: NonZeroU16,

    #[arg(long)]
    t: NonZeroU8,

    #[arg(long)]
    base_bit: NonZeroU8,

    #[arg(long, value_enum)]
    #[arg(default_value_t = OutputType::Accuracy)]
    output: OutputType,

    #[arg(long)]
    #[arg(default_value_t = available_parallelism().unwrap())]
    threads: NonZeroUsize,

    #[arg(long)]
    reencrypt: bool,

    #[arg(long)]
    balance: bool,

    #[arg(long)]
    verbose: bool,
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
enum ThermType {
    Linear,
    Log,
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
enum ActivationType {
    Binary,
    Linear,
    Log,
    BoundedLog,
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
enum OutputType {
    Accuracy,
    Predictions,
    Scores,
}

fn main() {
    let opts = <Opt as clap::Parser>::parse();

    if opts.verbose {
        eprintln!("Arguments: {opts:#?}");
    }

    if opts.sigma <= 0.0 || opts.sigma >= 1.0 {
        panic!("The sigma parameter must be between 0.0 and 1.0.");
    }

    if !opts.upper_n.is_power_of_two() {
        panic!("The upper_n parameter must be a power of two.");
    }

    if opts.upper_n.get() < 512 || opts.upper_n.get() > 4096 {
        panic!("The upper_n parameter must be between 512 and 4096.");
    }

    if opts.l.get() > 54 {
        panic!("The l parameter must be between 1 and 54.");
    }

    if opts.bg_bit.get() > 54 {
        panic!("The Bg_bit parameter must be between 1 and 54.");
    }

    if opts.l.get() * opts.bg_bit.get() > 54 {
        panic!("The product between l and Bg_bit must be between 1 and 54.");
    }

    if opts.t.get() > 54 {
        panic!("The t parameter must be between 1 and 54.");
    }

    if opts.base_bit.get() > 54 {
        panic!("The base_bit parameter must be between 1 and 54.");
    }

    if opts.t.get() * opts.base_bit.get() > 54 {
        panic!("The product between t and base_bit must be between 1 and 54.");
    }

    // Set the thread pool size
    rayon::ThreadPoolBuilder::new()
        .num_threads(opts.threads.get())
        .build_global()
        .unwrap();

    // Expand the seed to fill the 32 bytes
    let seed: [u8; 32] = opts
        .seed
        .to_le_bytes()
        .repeat(2)
        .as_slice()
        .try_into()
        .unwrap();

    // Configure the thermometer type and size
    let therm_size = opts.therm_size.get();
    let therm: Box<dyn SampleEncoder<u8, u8, _>> = match opts.therm_type {
        ThermType::Linear => {
            let therm = LinearThermometer::with_resolution(therm_size);
            Box::new(therm)
        }
        ThermType::Log => {
            let therm = LogThermometer::with_resolution(therm_size);
            Box::new(therm)
        }
    };

    // Get the number of bits used to represent data
    let data_bits = opts.data_bits.get() as usize;

    // Load and encode train samples
    let train_samples = wis_load_xz_csv(opts.train_data, data_bits)
        .unwrap()
        .into_iter()
        .skip(opts.train_skip as usize)
        .take(opts.train_limit as usize)
        .map(|sample| {
            let start = opts.data_skip;
            let limit = opts.data_limit.unwrap_or_else(|| {
                NonZeroU8::new(opts.data_bits.get()).unwrap()
            });
            let end = start + limit.get();
            let mut sample = Slice::new(start, end).encode(sample);
            therm.encode_inplace(&mut sample);
            <Permute>::with_seed(seed).encode_inplace(&mut sample);
            sample.set_vsize(opts.address_size.get() as usize);
            sample
        })
        .collect();
    let train_set = Dataset::from_samples(train_samples);

    // Calculate the label set size
    let label_set: HashSet<u8> = train_set.iter().map(|s| *s.label()).collect();
    let num_labels = opts
        .num_labels
        .or_else(|| NonZeroU8::new(label_set.len() as u8))
        .unwrap()
        .get() as usize;

    // Calculate the label distribution in train set
    let label_dist =
        train_set
            .iter()
            .fold(vec![0; num_labels], |mut acc, sample| {
                let label = *sample.label() as usize;
                acc[label] += 1;
                acc
            });

    // Find the most frequent label
    let most_freq_label = label_dist
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|(_, c)| *c)
        .unwrap();

    // Calculate the label weights
    let label_weights = label_dist
        .into_iter()
        .map(|count| most_freq_label.1 as f64 / count as f64)
        .collect::<Vec<_>>();

    // Display the label weights
    if opts.verbose {
        eprintln!("Label Weights: {label_weights:#?}");
    }

    // WiSARD parameters
    let input_size = train_set[0].len();
    let addr_size = opts.address_size.get() as usize;
    let num_luts_disc = (input_size + addr_size - 1) / addr_size;
    let label_size = num_labels.next_power_of_two().ilog2() as usize;
    let count_bits = opts.counter_size.get() as usize;
    let addr_label_size = addr_size + label_size;

    // Display WiSARD parameters
    if opts.verbose {
        eprintln!("Input size: {input_size}");
        eprintln!("Addr. size: {addr_size}");
        eprintln!("LUTs/label: {num_luts_disc}");
        eprintln!("Nr. labels: {num_labels}");
        eprintln!("Label size: {label_size}");
    }

    // TFHE parameters
    let sigma = opts.sigma;
    let base_bit = opts.base_bit.get() as u32;
    let upper_n = opts.upper_n.get() as u32;
    let bg_bit = opts.bg_bit.get() as u32;
    let l = opts.l.get() as u32;
    let t = opts.t.get() as u32;
    let k = 1;

    // Generate new keys
    let trlwe_key = TrlweKey::new(upper_n, k, sigma);
    let trgsw_key = TrgswKey::new(&trlwe_key, l, bg_bit);
    let tlwe_key = TlweKey::from_trlwe_key(&trlwe_key);
    let pks_key = TrlwePKSKey::new(&tlwe_key, &trlwe_key, t, base_bit);

    // Calculate useful information
    let log_upper_n = upper_n.ilog2() as usize;
    let lut_vp_depth = addr_label_size.saturating_sub(log_upper_n);
    let lut_vp_count = 1usize << lut_vp_depth;
    let trn_low_size = addr_label_size.min(log_upper_n);
    let inf_low_size = addr_size.min(log_upper_n);
    let lower_label_bits = label_size.saturating_sub(lut_vp_depth);
    let upper_label_bits = label_size - lower_label_bits;
    let upper_lut_size = 1 << (lut_vp_depth - upper_label_bits);
    let lower_lut_size = 1 << (trn_low_size - lower_label_bits);

    if opts.verbose {
        eprintln!("V.P. depth: {lut_vp_depth}");
        eprintln!("V.P. count: {lut_vp_count}");
        eprintln!("Upper bits: {upper_label_bits}");
        eprintln!("Lower bits: {lower_label_bits}");
        eprintln!("Upper size: {upper_lut_size}");
        eprintln!("Lower size: {lower_lut_size}");
        eprintln!();
    }

    // Transform the dataset into label + index + address pairs
    let train_addresses = wis_gen_addrs(&train_set);
    drop(train_set);

    // Instantiate thread local storage
    let tls = ThreadLocal::new();

    // Get the total number of training addresses
    let train_addr_count = train_addresses.len();

    // Define a progress bar template
    let bar_template = concat! {
        "[{elapsed_precise}] ",
        "{bar:50.cyan/blue} ",
        "{pos}/{len} ",
        "(eta: {eta_precise})\n",
    };

    // Define a progress bar style
    let bar_style = ProgressStyle::with_template(bar_template)
        .unwrap()
        .progress_chars("##-");

    // Instantiate a progress bar
    let bar = ProgressBar::new(train_addr_count as u64)
        .with_style(bar_style.clone())
        .with_finish(ProgressFinish::Abandon);

    // Allocate additional Vecs to store iteration times
    #[cfg(feature = "time-tracking")]
    let (cl_times, sv_times) = (AtomicU64::default(), AtomicU64::default());

    eprintln!("Training {} samples...", train_addr_count / num_luts_disc);

    // Store wall time
    let wall_begin = Instant::now();

    // Instantiate a parallel iterator over the train addresses
    let train_iter = train_addresses.into_par_iter().progress_with(bar);

    // Train the encrypted LUTs in parallel
    train_iter.for_each(|(label, index, addr)| {
        // Measure client iteration time
        #[cfg(feature = "time-tracking")]
        let begin = Instant::now();

        // Convert index to usize
        let index = index as usize;

        // Generate and encrypt a zeroed LUT containing one on first index
        let mut enc_mask = TrlweArray::from_fn(lut_vp_count, &trlwe_key, |i| {
            TorusPolynomial::from_fn(upper_n, |j| {
                let value = i == 0 && j == 0;
                Torus::from_unsigned(value as RawTorus, count_bits)
            })
        });

        // Concatenate and encrypt address and label
        let addr_label = ((label as usize) << addr_size) | (addr as usize);
        let enc_addr_label =
            TrgswDftArray::from_fn(addr_label_size, &trgsw_key, |i| {
                (Torus::from_raw((addr_label as RawTorus >> i) & 1), 0)
            });

        // Store elapsed client time
        #[cfg(feature = "time-tracking")]
        {
            let elapsed = begin.elapsed().as_micros() as u64;
            cl_times.fetch_add(elapsed, Ordering::Relaxed);
        }

        // Instantiate thread-local LUTs
        let enc_luts = tls.get_or(|| {
            let zlut = TrlweArray::zeroed_noiseless(lut_vp_count, k, upper_n);
            RefCell::new(vec![zlut; num_luts_disc])
        });

        // Split the bits for blind-rotation and cdemux-tree
        let (lower, upper) = enc_addr_label.as_slice().split_at(trn_low_size);

        // Measure server iteration time
        #[cfg(feature = "time-tracking")]
        let begin = Instant::now();

        // Apply a left-handed blind rotation
        for (i, bit) in lower.iter().enumerate().rev() {
            // Compute the rotation of the polynomial by 2^i
            let rot = enc_mask[0].mul_by_xai_pred(1 << i);

            // Conditionally apply the rotation based on the selector bit
            let rot = Trlwe::from_dft(&TrlweDft::mul_trlwe_dft(&rot, bit));

            // Apply the rotation over the original LUT
            enc_mask[0].add_assign(&rot);
        }

        // Apply the CDEMUX tree
        for (i, bit) in upper.iter().enumerate().rev() {
            // Apply CDEMUX between an strided vector of elements
            bit.cdemux_vectored(enc_mask.as_slice_mut(), 1 << i);
        }

        // Train the LUTs using the mask
        let mut enc_luts = enc_luts.borrow_mut();
        for (lut, mask) in enc_luts[index].iter_mut().zip(enc_mask.iter()) {
            lut.add_assign(mask);
        }

        // Store elapsed server time
        #[cfg(feature = "time-tracking")]
        {
            let elapsed = begin.elapsed().as_micros() as u64;
            sv_times.fetch_add(elapsed, Ordering::Relaxed);
        }
    });

    // Finish wall time measurement
    let wall_elapsed = wall_begin.elapsed().as_millis() as u64;
    eprintln!("Training complete. Elapsed time (ms): {wall_elapsed}.\n");

    #[cfg(feature = "time-tracking")]
    if opts.verbose {
        // Calculate average times
        let addr_count = train_addr_count as u64;
        let cl_times = cl_times.into_inner();
        let sv_times = sv_times.into_inner();
        let cl_iter = (cl_times + (addr_count >> 1)) / addr_count;
        let sv_iter = (sv_times + (addr_count >> 1)) / addr_count;

        // Display average / total times
        eprintln!("--------------------------------------------------------");
        eprintln!("  Client Training Time (us): {cl_times}");
        eprintln!("     * Avg. Iter. Time (us): {cl_iter}");
        eprintln!("  Server Training Time (us): {sv_times}");
        eprintln!("     * Avg. Iter. Time (us): {sv_iter}");
        eprintln!("--------------------------------------------------------\n");
    }

    // Combine thread local results
    let zlut = TrlweArray::zeroed_noiseless(lut_vp_count, k, upper_n);
    let mut enc_luts =
        tls.into_iter()
            .fold(vec![zlut; num_luts_disc], |mut accs, luts| {
                let iter = accs.iter_mut().zip(luts.into_inner());
                for (accs, luts) in iter {
                    for (acc, lut) in accs.iter_mut().zip(luts.iter()) {
                        acc.add_assign(lut);
                    }
                }
                accs
            });

    // Decrypt, process, and encrypt back, if user requested
    if opts.reencrypt {
        eprintln!("Performing re-encryption...");
        enc_luts = enc_luts
            .into_iter()
            .map(|luts| {
                TrlweArray::from_fn(lut_vp_count, &trlwe_key, |i| {
                    let poly = luts[i].phase(&trlwe_key);
                    TorusPolynomial::from_fn(upper_n, |i| {
                        Torus::from_unsigned(
                            poly[i].into_unsigned(count_bits),
                            count_bits,
                        )
                    })
                })
            })
            .collect();

        eprintln!("Re-encryption complete.\n")
    }

    // Load and encode test samples
    let test_samples = wis_load_xz_csv(opts.test_data, data_bits)
        .unwrap()
        .into_iter()
        .skip(opts.test_skip as usize)
        .take(opts.test_limit as usize)
        .map(|sample| {
            let start = opts.data_skip;
            let limit = opts.data_limit.unwrap_or_else(|| {
                NonZeroU8::new(opts.data_bits.get()).unwrap()
            });
            let end = start + limit.get();
            let mut sample = Slice::new(start, end).encode(sample);
            therm.encode_inplace(&mut sample);
            <Permute>::with_seed(seed).encode_inplace(&mut sample);
            sample.set_vsize(addr_size);
            sample
        })
        .collect();
    let test_set = Dataset::from_samples(test_samples);

    // Transform the dataset into label + index + address pairs
    let test_addresses = wis_gen_addrs(&test_set);
    let test_labels: Vec<u8> = test_set.iter().map(|s| *s.label()).collect();
    drop(test_set);

    // Get the total number of testing addresses and result chunks
    let test_addr_count = test_addresses.len();
    let num_results = num_labels * test_addr_count;
    let num_chunks = (num_results + upper_n as usize - 1) / upper_n as usize;

    // Instantiate a progress bar
    let bar = ProgressBar::new(num_results as u64)
        .with_style(bar_style.clone())
        .with_finish(ProgressFinish::Abandon);

    // Allocate a TRLWE array for the chunks
    let mut enc_chunks = TrlweArray::zeroed_noiseless(num_chunks, k, upper_n);

    #[cfg(feature = "noise-tracking")]
    let (trn_var, inf_var, pks_var): (AtomicF64, AtomicF64, AtomicF64) =
        Default::default();

    #[cfg(feature = "time-tracking")]
    let (cl_times, sv_times, ks_times): (AtomicU64, AtomicU64, AtomicU64) =
        Default::default();

    eprintln!("Evaluating {} samples...", test_addr_count / num_luts_disc);

    // Store wall time
    let wall_begin = Instant::now();

    // Instantiate a parallel iterator over the test addresses
    let enc_chunks_iter = enc_chunks.as_slice_mut().chunks_mut(num_labels);
    let test_iter =
        test_addresses.chunks(upper_n as usize).zip(enc_chunks_iter);

    // Evaluate the encrypted LUTs
    test_iter.for_each(|(chunk, chunk_results)| {
        // Allocate a TLWE array for the chunk results
        let tlwe_rsize = chunk.len() * num_labels;
        let mut tlwe_results = TlweArray::zeroed_noiseless(tlwe_rsize, upper_n);

        #[cfg(feature = "noise-tracking")]
        let mut den_results = vec![Torus::MIN; tlwe_rsize];

        #[cfg(feature = "noise-tracking")]
        let tlwe_iter = tlwe_results
            .as_slice_mut()
            .par_chunks_mut(num_labels)
            .zip_eq(den_results.par_chunks_mut(num_labels));

        #[cfg(not(feature = "noise-tracking"))]
        let tlwe_iter = tlwe_results.as_slice_mut().par_chunks_mut(num_labels);

        // Instantiate an iterator over label set sized chunks
        let chunk_iter = chunk.into_par_iter().zip_eq(tlwe_iter);

        // Iterate over all the samples of a chunk
        chunk_iter.for_each(|(&(_, index, addr), sample_results)| {
            // Measure iteration time
            #[cfg(feature = "time-tracking")]
            let begin = Instant::now();

            // Encrypt only the address bits (without label)
            let enc_addr = TrgswDftArray::from_fn(addr_size, &trgsw_key, |i| {
                (Torus::from_raw((addr as RawTorus >> i) & 1), 0)
            });

            // Store elapsed time
            #[cfg(feature = "time-tracking")]
            {
                let elapsed = begin.elapsed().as_micros() as u64;
                cl_times.fetch_add(elapsed, Ordering::Relaxed);
            }

            // Convert index to usize
            let index = index as usize;

            // Split the selectors for blind-rotation and cmux-tree
            let (lower, upper) = enc_addr.as_slice().split_at(inf_low_size);

            #[cfg(feature = "noise-tracking")]
            let sample_iter = sample_results
                .0
                .par_iter_mut()
                .zip_eq(sample_results.1.par_iter_mut())
                .enumerate();

            #[cfg(not(feature = "noise-tracking"))]
            let sample_iter = sample_results.par_iter_mut().enumerate();

            // Expand index-address tuples for each label
            sample_iter.for_each(|(label, result)| {
                // Measure iteration time
                #[cfg(feature = "time-tracking")]
                let begin = Instant::now();

                // Separate upper and lower label bits
                let upper_label = label >> lower_label_bits;
                let lower_label = label & ((1 << lower_label_bits) - 1);

                // Calculate upper and lower LUT offsets
                let upper_offset = upper_label * upper_lut_size;
                let lower_offset = lower_label * lower_lut_size;

                // Clone the original LUT for in-place vertical packing
                let mut lut = enc_luts[index].as_slice()
                    [upper_offset..upper_offset + upper_lut_size]
                    .to_vec();

                // Apply the CMUX tree
                for (i, bit) in upper.iter().enumerate() {
                    // Apply CMUX between an strided vector of elements
                    bit.cmux_vectored(lut.as_mut_slice(), 1 << i);
                }

                // Apply a right-handed blind rotation
                for (i, bit) in lower.iter().enumerate() {
                    // Compute the rotation of the polynomial by 2N - 2^i
                    let after = lut[0].mul_by_xai_pred(upper_n * 2 - (1 << i));

                    // Apply the rotation based on the selector bit
                    let trlwe_dft = TrlweDft::mul_trlwe_dft(&after, bit);
                    let after = Trlwe::from_dft(&trlwe_dft);

                    lut[0].add_assign(&after);
                }

                #[cfg(feature = "noise-tracking")]
                result.0.set_from_trlwe(&lut[0], lower_offset);

                #[cfg(not(feature = "noise-tracking"))]
                result.set_from_trlwe(&lut[0], lower_offset);

                #[cfg(feature = "time-tracking")]
                {
                    // Store elapsed time
                    let elapsed = begin.elapsed().as_micros() as u64;
                    sv_times.fetch_add(elapsed, Ordering::Relaxed);
                }

                #[cfg(feature = "noise-tracking")]
                {
                    // Calculate lower and upper parts of the address
                    let addr_label = (label << addr_size) | addr as usize;
                    let up_addr = addr_label >> log_upper_n;
                    let lo_addr = addr_label & (upper_n as usize - 1);

                    // Obtain the original LUT
                    let original_lut = &enc_luts[index][up_addr];

                    // Extract the value before processing
                    let bef = &Tlwe::from_trlwe(original_lut, lo_addr);
                    let before_dec =
                        bef.phase(&tlwe_key).into_unsigned(count_bits);
                    let before_dec =
                        Torus::from_unsigned(before_dec, count_bits);

                    // Calculate the deviation before processing
                    let after_dec = bef.phase(&tlwe_key);
                    let var =
                        after_dec.distance(before_dec).into_double().powi(2);
                    trn_var.fetch_add(var, Ordering::Relaxed);

                    // Calculate the deviation after processing
                    let after_dec = result.0.phase(&tlwe_key);
                    let var =
                        after_dec.distance(before_dec).into_double().powi(2);
                    inf_var.fetch_add(var, Ordering::Relaxed);

                    // Save the denoised result for key switching
                    *result.1 = before_dec;
                }

                // Increment the progress bar
                bar.inc(1);
            });
        });

        #[cfg(feature = "noise-tracking")]
        let keyswitch_iter = chunk_results
            .into_par_iter()
            .zip_eq(den_results.par_chunks_mut(upper_n as usize))
            .enumerate();

        #[cfg(not(feature = "noise-tracking"))]
        let keyswitch_iter = chunk_results.into_par_iter().enumerate();

        // Iterate and apply packing key switch
        keyswitch_iter.for_each(|(i, result)| {
            let skip = i * upper_n as usize;
            let offset = (tlwe_results.len() - skip).min(upper_n as usize);

            #[cfg(feature = "noise-tracking")]
            let (result, den_result) = result;

            // Measure iteration time
            #[cfg(feature = "time-tracking")]
            let begin = Instant::now();

            result.set_from_tlwe_array(&tlwe_results, skip, offset, &pks_key);

            // Store elapsed time
            #[cfg(feature = "time-tracking")]
            {
                let elapsed = begin.elapsed().as_micros() as u64;
                ks_times.fetch_add(elapsed, Ordering::Relaxed);
            }

            #[cfg(feature = "noise-tracking")]
            {
                let after_len = den_result.len().min(upper_n as usize);
                let after_dec = result.phase(&trlwe_key);
                let var = den_result
                    .iter()
                    .zip(after_dec.iter().take(after_len))
                    .map(|(den, aft)| den.distance(*aft).into_double().powi(2))
                    .sum::<f64>();
                pks_var.fetch_add(var, Ordering::Relaxed);
            }
        });
    });

    // Finish the progress bar
    bar.finish();

    // Display elapsed time
    let wall_elapsed = wall_begin.elapsed().as_millis() as u64;
    eprintln!("Evaluation complete. Elapsed time (ms): {wall_elapsed}.\n");

    #[cfg(feature = "time-tracking")]
    if opts.verbose {
        // Calculate average times
        let addr_count = test_addr_count as u64;
        let num_chunks = num_chunks as u64;
        let cl_times = cl_times.into_inner();
        let sv_times = sv_times.into_inner();
        let ks_times = ks_times.into_inner();
        let cl_iter = (cl_times + (addr_count >> 1)) / addr_count;
        let sv_iter = (sv_times + (addr_count >> 1)) / addr_count;
        let ks_iter = (ks_times + (num_chunks >> 1)) / num_chunks;

        // Display average / total times
        eprintln!("--------------------------------------------------------");
        eprintln!(" Client Inference Time (us): {cl_times}");
        eprintln!("     * Avg. Iter. Time (us): {cl_iter}");
        eprintln!(" Server Inference Time (us): {sv_times}");
        eprintln!("     * Avg. Iter. Time (us): {sv_iter}");
        eprintln!(" Server Keyswitch Time (us): {ks_times}");
        eprintln!("     * Avg. Iter. Time (us): {ks_iter}");
        eprintln!("--------------------------------------------------------\n");
    }

    // Decrypt the result chunks
    let raw_results: Vec<Torus> = enc_chunks
        .iter()
        .flat_map(|chk| {
            chk.phase(&trlwe_key).iter().cloned().collect::<Vec<_>>()
        })
        .take(num_results)
        .collect();

    #[cfg(feature = "noise-tracking")]
    if opts.verbose {
        // Calculate average variances
        let trn_var = trn_var.into_inner() / num_results as f64;
        let inf_var = inf_var.into_inner() / num_results as f64;
        let pks_var = pks_var.into_inner() / num_results as f64;

        // Display post-training, post-inference, and post-keyswitch variances
        eprintln!("--------------------------------------------------------");
        eprintln!("     Post-Training Variance: {trn_var:e}");
        eprintln!("    Post-Inference Variance: {inf_var:e}");
        eprintln!("    Post-Keyswitch Variance: {pks_var:e}");
        eprintln!("--------------------------------------------------------\n");
    }

    // Calculate the scores
    let scores: Vec<Vec<RawTorus>> = raw_results
        .chunks(num_labels * num_luts_disc)
        .map(|sval| {
            let sums = vec![0 as RawTorus; num_labels];
            sval.chunks(num_labels).fold(sums, |mut sums, scores| {
                sums.iter_mut().zip(scores.iter()).enumerate().for_each(
                    |(label, (sum, v))| {
                        // Denoise and apply the activation function over the value
                        let den_v = v.into_unsigned(count_bits);
                        let bal_v = match opts.balance {
                            false => den_v,
                            true => {
                                (den_v as f64 * label_weights[label])
                                    as RawTorus
                            }
                        };
                        let thr_v = bal_v.saturating_sub(opts.threshold);
                        let act_v = match opts.activation {
                            ActivationType::Binary => (thr_v > 0).into(),
                            ActivationType::Linear => thr_v,
                            ActivationType::Log => (thr_v + 1).ilog2().into(),
                            ActivationType::BoundedLog => {
                                ((thr_v + 1).ilog2() as RawTorus).min(5)
                            }
                        };

                        // Sum the activated value into the final score
                        *sum += act_v;
                    },
                );
                sums
            })
        })
        .collect();

    // Display scores and exit, if requested
    if let OutputType::Scores = opts.output {
        println!("{scores:?}");
        return;
    }

    // Predict based on maximum scores
    let predictions: Vec<u8> = scores
        .into_iter()
        .map(|scores| {
            scores
                .into_iter()
                .enumerate()
                .map(|(pred, score)| (pred as u8, score))
                .max_by_key(|&(_, score)| score)
                .map(|(pred, _)| pred)
                .unwrap()
        })
        .collect();

    // Display predictions and exit, if requested
    if let OutputType::Predictions = opts.output {
        println!("{predictions:?}");
        return;
    }

    // Count number of labels in the test set
    let num_samples = test_labels.len();
    let total_per_label = test_labels.iter().copied().fold(
        vec![0usize; num_labels],
        |mut acc, label| {
            acc[label as usize] += 1;
            acc
        },
    );

    // Calculate number of correct predictions per label
    let correct_per_label = predictions
        .into_iter()
        .zip(test_labels)
        .filter_map(|(pred, truth)| (pred == truth).then_some(pred))
        .fold(vec![0usize; num_labels], |mut acc, label| {
            acc[label as usize] += 1;
            acc
        });

    // Calculate accuracy per label
    let accuracy_per_label = correct_per_label
        .iter()
        .copied()
        .zip(total_per_label)
        .map(|(correct, total)| correct as f64 / total as f64)
        .collect::<Vec<_>>();

    // Display the accuracy per label
    println!("Label Accuracy: {accuracy_per_label:?}\n");

    // Calculate the final accuracy
    let total_correct = correct_per_label.into_iter().sum::<usize>();
    let accuracy = total_correct as f64 / num_samples as f64;

    // Display the final accuracy
    println!("Accuracy: {:.2}%", accuracy * 100.0);
}
