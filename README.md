# Homomorphic WiSARDs

This repository contains the code to accompany the paper *Homomorphic WiSARDs:
Efficient Weightless Neural Network training over encrypted data*. The repository
is organized into three projects:
-  `mosfhet` contains an implementation of the TFHE scheme (an extended RUST version of [MOSFHET](https://github.com/antoniocgj/MOSFHET))
- `wisard-he` is the Homomorphic WiSARD implementation, and
- `wisard-pt` is the equivalent cleartext implementation of `wisard-he`.

### Citation

```
TBD
```

## Requirements

Our codebase has the following dependencies:

- Rust 1.66.1 or later;
- Clang or GCC C toolchain;

Using compatible Clang and LLD is recommended to make use of the cross-language LTO optimization between C and Rust.

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

WiSARD options:

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
| --output | Type of output – accuracy/predictions/scores (default: accuracy) |
| --help | Displays a help message |
| --threads | Number of threads to use |

HE options:

| Option | Description|
|:------:|:-------:|
| --sigma | Sigma (noise std. dev.) for RLWE/RGSW encryption |
| --upper-n | N parameter (dimension) for RLWE/RGSW encryption |
| --l | ℓ (\ell) parameter for RGSW samples |
| --bg-bit | log<sub>2</sub>(β) parameter for RGSW samples |
| --t | T parameter for TFHE keyswitching |
| --base-bit | Base_bit parameter for TFHE keyswitching |
| --reencrypt | Activates re-encryption after training |

## License

- `mosfhet`: [Apache License (Version 2.0)](./LICENSE-Apache). See [detailed copyright information](https://github.com/leonardohn/homomorphic-wisards/tree/main/mosfhet/mosfhet-sys/MOSFHET#license).
- `wisard-he` and `wisard-pt`: [MIT](./LICENSE-MIT) and [Apache License (Version 2.0)](./LICENSE-Apache)
- Datasets: See each folder.
