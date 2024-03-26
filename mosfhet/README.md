# `mosfhet`

This Rust crate provides safe bindings for the [MOSFHET][MOSFHET] C library, a
research-oriented, highly-optimized implementation of the [TFHE][TFHE]
homomorphic encryption scheme. These bindings provide a type-safe, memory-safe
interface that leverages Rust's ownership and concurrency models to further
assist the developer while retaining similar performance and memory footprint.

[MOSFHET]: https://github.com/antoniocgj/MOSFHET.git
[TFHE]: https://eprint.iacr.org/2018/421.pdf

## Installation

Add `mosfhet` to your Rust project by including it in your `Cargo.toml` file:

```toml
[dependencies]
mosfhet = "0.1.0"
```

The default configuration will provide a portable, generic build. For the best
performance on a x86-64 CPU with AVX-512 + VAES support, we suggest enabling the
following features:

```toml
[dependencies.mosfhet]
version = "0.1.0"
default-features = false
features = [
  "fft_spqlios",
  "fft_spqlios_avx512",
  "rng_vaes",
]
```

The features available are as follows:

* `bindgen`: use `bindgen` to generate bindings;
* `fft_ffnt`: use `ffnt` as the FFT library (portable);
* `fft_ffnt_fma`: enable FMA support for the `ffnt` library (x86);
* `fft_spqlios`: use `spqlios` as the FFT library (x86);
* `fft_spqlios_avx512`: enable AVX-512 support for the `spqlios` library (x86);
* `rng_shake`: use `SHAKE` algorithm as the RNG source (secure);
* `rng_xoshiro`: use `xoshiro` algorithm as the RNG source (insecure);
* `rng_vaes`: use `VAES` CPU extension as the RNG source (experimental);
* `torus32`: use compact 32-bit torus (untested);

## Usage

Usage examples may be found at the [examples](examples/) directory.

## Contribution

Contributions to the `mosfhet` project are welcome! If you find a bug or have
suggestions for improvements, please open an
[Issue](https://github.com/leonardohn/mosfhet/issues). 
[Pull requests](https://github.com/leonardohn/mosfhet/pulls) for new features,
bug fixes, and documentation enhancements are also appreciated.

## License

`mosfhet` is distributed under the terms of the 
[Apache License (Version 2.0)](LICENSE).
