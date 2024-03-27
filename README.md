# Homomorphic WiSARDs

This repository contains the code to accompany the paper *Homomorphic WiSARDs:
Efficient Weightless Neural Network training over encrypted data*. The repository
is organized into three projects:
-  `mosfhet` contains an implementation of TFHE scheme (an extended RUST version of [MOSFHET](https://github.com/antoniocgj/MOSFHET))
- `wisard-he` the Homomorphic WiSARD implementation, and
- `wisard-pt` the equivalent cleartext implementation of `wisard-he`.

## Requirements

Our codebase have the following dependencies:

- Rust 1.66.1 or later;
- Clang or GCC C toolchain;

Using compatible Clang and LLD is recommended to make use of the cross-language
LTO optimization between C and Rust.

## Usage

### Running pre-configured experiments

To run the encrypted model training and evaluation:

```bash
cd wisard-he
../scripts/run-<param set name>.sh
```

Where `<param set name>` is replaced by the available parameter set names.

To run the cleartext model training and evaluation:

```bash
cd wisard-pt
../scripts/run-<param set name>.sh
```

### Command line arguments

| Option | Description|
|:------:|:-------:|
| --num-labels  | Number of labels to allocate (if not provided, it infers from the samples) |
| --data-bits   | Number of bits to represent the data (default: 8) |
| --data-limit  | Number of bits to consider in each data value (default: 8) |
| --data-skip   | Number of bits to skip at the start of each data value  (default: 0)|
| --train-data  | Path to the training dataset |
| --train-limit | Number of samples to consider from the training dataset |
| --train-skip | Number of samples to skip at the beginning of the training dataset |
| --test-data | Path to the testing dataset |
| --test-limit | Number of samples to consider from the testing dataset |
| --test-skip | Number of samples to skip at the beginning of the testing dataset |
| --address-size | Number of bits in the address |
| --counter-size | Number of bits in the counters |
| --therm-size | Number of bits in the thermometer |
| --therm-type | Type of thermometer – linear/log (default: linear) |
| --activation | Type of activation – binary/linear/logarithmic/bounded-log (default: binary) |
| --threshold | Threshold / bleaching value |
| --seed | Seed for randomness |
| --sigma | Sigma parameter for TFHE |
| --l | \ell parameter for TFHE |
| --bg-bit | Bg_bit parameter for TFHE |
| --upper-n | N parameter for TFHE |
| --t | T parameter for TFHE |
| --base-bit | Base_bit parameter for TFHE |
| --output | Type of output – accuracy/predictions/scores (default: accuracy) |
| --threads | Number of threads to use |
| --reencrypt | Activates re-encryption after training |
| --help | Displays a help message |

