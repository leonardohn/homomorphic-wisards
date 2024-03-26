use bitvec::prelude::*;
use wisard::dataset::Dataset;
use wisard::sample::{Label, Sample};

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
                    // Should panic if the size is bigger than what can be handled
                    let index = u16::try_from(index).unwrap();
                    let addr = u32::try_from(addr).unwrap();
                    let label = *samp.label();
                    (label, index, addr)
                })
        })
        .collect()
}
