use std::collections::HashSet;
use std::num::{NonZeroU16, NonZeroU8, NonZeroUsize};
use std::path::PathBuf;
use std::thread::available_parallelism;

use bitvec::prelude::*;
use wisard::dataset::Dataset;
use wisard::encode::{
    LinearThermometer, LogThermometer, Permute, SampleEncoder, Slice,
};
use wisard::sample::{Label, Sample};

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
    threshold: u32,

    #[arg(long)]
    #[arg(default_value_t = rand::random())]
    seed: u128,

    #[arg(long)]
    sigma: Option<f64>,

    #[arg(long)]
    l: Option<NonZeroU8>,

    #[arg(long)]
    bg_bit: Option<NonZeroU8>,

    #[arg(long)]
    upper_n: Option<NonZeroU16>,

    #[arg(long)]
    t: Option<NonZeroU8>,

    #[arg(long)]
    base_bit: Option<NonZeroU8>,

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

    #[arg(short, long)]
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

pub type SampleLoadResult =
    Result<Vec<Sample<u8, u8, Lsb0>>, Box<dyn std::error::Error>>;

// Load XZipped CSV samples from file
pub fn wis_load_xz_csv<P>(path: P, data_size: usize) -> SampleLoadResult
where
    P: AsRef<std::path::Path>,
{
    // Load the file, unzip the contents, and parse the records
    csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(xz2::read::XzDecoder::new(std::fs::File::open(path)?))
        .into_records()
        .map(|rec| {
            // Convert each record into a bit vector
            rec.map_err(|e| e.into()).and_then(|rec| {
                let mut iter = rec.into_iter().map(str::parse::<u8>);
                let label = iter.next().unwrap().unwrap();
                iter.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.into())
                    .map(BitVec::from_vec)
                    .map(|v| Sample::from_raw_parts(v, data_size, label))
            })
        })
        .collect()
}

// Generate WiSARD address pairs from samples
pub fn wis_gen_addrs<L, T, O>(dset: &Dataset<L, T, O>) -> Vec<(L, u16, u32)>
where
    L: Label,
    T: BitStore + serde::de::DeserializeOwned,
    T::Mem: serde::Serialize,
    O: BitOrder,
{
    // Iterate through samples
    dset.iter()
        .flat_map(|samp| {
            // Iterate through sample values (groups of bits)
            samp.iter_values()
                .map(|addr| {
                    // Concatenate all the sample bits
                    addr.iter().enumerate().fold(0, |addr, (index, bit)| {
                        addr | ((*bit as usize) << index)
                    })
                })
                .enumerate()
                .map(|(index, addr)| {
                    // Should panic if the size is bigger than the variable
                    let index = u16::try_from(index).unwrap();
                    let addr = u32::try_from(addr).unwrap();
                    let label = *samp.label();
                    (label, index, addr)
                })
        })
        .collect()
}

fn main() {
    let opts = <Opt as clap::Parser>::parse();

    if opts.verbose {
        eprintln!("Arguments: {opts:#?}");
    }

    // Expand the seed to fill the 32 bytes
    let seed: [u8; 32] = opts
        .seed
        .to_le_bytes()
        .repeat(2)
        .as_slice()
        .try_into()
        .unwrap();

    // Configure the thermometer type and size
    let therm_size = opts.therm_size;
    let therm: Box<dyn SampleEncoder<u8, u8, _>> = match opts.therm_type {
        ThermType::Linear => {
            let therm = LinearThermometer::with_resolution(therm_size.get());
            Box::new(therm)
        }
        ThermType::Log => {
            assert!(therm_size.is_power_of_two());
            let therm = LogThermometer::with_resolution(therm_size.get());
            Box::new(therm)
        }
    };

    // Load and encode train samples
    let train_set = Dataset::from_samples(
        wis_load_xz_csv(opts.train_data, opts.data_bits.get() as usize)
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
            .collect(),
    );

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
        eprintln!("Class Weights: {label_weights:?}");
    }

    // Load and encode test samples
    let test_set = Dataset::from_samples(
        wis_load_xz_csv(opts.test_data, opts.data_bits.get() as usize)
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
                sample.set_vsize(opts.address_size.get() as usize);
                sample
            })
            .collect(),
    );

    // WiSARD parameters
    let input_size = train_set[0].len();
    let addr_size = opts.address_size.get() as usize;
    let num_luts_disc = (input_size + addr_size - 1) / addr_size;

    // Display WiSARD parameters
    if opts.verbose {
        eprintln!("Input size: {input_size}");
        eprintln!("Addr. size: {addr_size}");
        eprintln!("LUTs/label: {num_luts_disc}");
        eprintln!("Nr. Labels: {num_labels}");
    }

    // Transform the dataset into label + index + address pairs
    let train_addresses = wis_gen_addrs(&train_set);
    let test_addresses = wis_gen_addrs(&test_set);
    let test_labels: Vec<u8> = test_set.iter().map(|s| *s.label()).collect();
    drop(train_set);
    drop(test_set);

    // Allocate an appropriate LUT size
    let full_lut_size = (num_labels * num_luts_disc) << addr_size;
    let mut luts = vec![0u32; full_lut_size];

    // Train the model using the training set
    for (label, index, addr) in train_addresses {
        let label = label as usize;
        let index = index as usize;
        let addr = addr as usize;
        let offset = (label * num_luts_disc + index) << addr_size;
        luts[offset | addr] += 1;
    }

    // Display the max value found in counters
    if opts.verbose {
        let max_count = luts.iter().max().unwrap();
        eprintln!("Max. count: {max_count}");
    }

    // Evaluate the model using the testing set
    let num_samples = test_labels.len();
    let max_count = 1 << opts.counter_size.get();
    let scores = test_addresses
        .into_iter()
        .map(|(_, index, addr)| {
            (0..num_labels)
                .map(|label| {
                    let index = index as usize;
                    let addr = addr as usize;
                    let offset = label * num_luts_disc + index;
                    let off_addr = (offset << addr_size) | addr;
                    let act = luts[off_addr] & (max_count - 1);
                    let act = match opts.balance {
                        false => act,
                        true => (act as f64 * label_weights[label]) as u32,
                    };
                    let a = act.saturating_sub(opts.threshold);
                    match opts.activation {
                        ActivationType::Binary => (a > 0) as u32,
                        ActivationType::Linear => a,
                        ActivationType::Log => (a + 1).ilog2(),
                        ActivationType::BoundedLog => (a + 1).ilog2().min(5),
                    }
                })
                .collect::<Vec<_>>()
        })
        .enumerate()
        .fold(
            vec![vec![0usize; num_labels]; num_samples],
            |mut acc, (i, scores)| {
                acc[i / num_luts_disc]
                    .iter_mut()
                    .zip(scores)
                    .for_each(|(acc, score)| *acc += score as usize);
                acc
            },
        );

    // Display scores and exit, if requested
    if let OutputType::Scores = opts.output {
        println!("{scores:?}");
        return;
    }

    // Make the predictions based on the scores
    let predictions = scores
        .iter()
        .map(|scores| {
            scores
                .iter()
                .enumerate()
                .max_by_key(|(_, score)| *score)
                .unwrap()
                .0 as u8
        })
        .collect::<Vec<_>>();

    // Display predictions and exit, if requested
    if let OutputType::Predictions = opts.output {
        println!("{predictions:?}");
        return;
    }

    // Count number of labels in the test set
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
