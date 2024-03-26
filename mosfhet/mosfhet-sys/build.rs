fn main() {
    #[cfg(all(feature = "fft_ffnt", feature = "fft_spqlios"))]
    compile_error!(
        "features \"fft_ffnt\" and \"fft_spqlios\" are mutually exclusive"
    );
    #[cfg(not(any(feature = "fft_ffnt", feature = "fft_spqlios")))]
    compile_error!(
        "either feature \"fft_ffnt\" or \"fft_spqlios\" must be enabled"
    );
    #[cfg(all(feature = "rng_shake", feature = "rng_vaes"))]
    compile_error!(
        "features \"rng_shake\" and \"rng_vaes\" are mutually exclusive"
    );
    #[cfg(all(feature = "rng_shake", feature = "rng_xoshiro"))]
    compile_error!(
        "feature \"rng_shake\" and \"rng_xoshiro\" are mutually exclusive"
    );
    #[cfg(all(feature = "rng_vaes", feature = "rng_xoshiro"))]
    compile_error!(
        "feature \"rng_vaes\" and \"rng_xoshiro\" are mutually exclusive"
    );
    #[cfg(not(any(
        feature = "rng_shake",
        feature = "rng_vaes",
        feature = "rng_xoshiro",
    )))]
    compile_error!(concat!(
        "either feature \"rng_shake\" or \"rng_vaes\"",
        "or \"rng_xoshiro\" must be enabled",
    ));

    #[cfg(feature = "bindgen")]
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut build = cc::Build::new();

    build
        .include("./MOSFHET/include")
        .file("./MOSFHET/src/keyswitch.c")
        .file("./MOSFHET/src/bootstrap.c")
        .file("./MOSFHET/src/bootstrap_ga.c")
        .file("./MOSFHET/src/tlwe.c")
        .file("./MOSFHET/src/trlwe.c")
        .file("./MOSFHET/src/trgsw.c")
        .file("./MOSFHET/src/misc.c")
        .file("./MOSFHET/src/polynomial.c")
        .file("./MOSFHET/src/register.c")
        .file("./MOSFHET/src/sha3/fips202.c")
        .file("./MOSFHET/src/fft/karatsuba.c")
        .file("./MOSFHET/src/rnd/aes_rng.c")
        .flag("-maes")
        .flag("-mavx")
        .flag("-mavx2")
        .flag("-mbmi2")
        .flag("-mrdrnd");

    #[cfg(feature = "fft_ffnt")]
    build.file("./MOSFHET/src/fft/ffnt/ffnt.c");

    #[cfg(feature = "fft_ffnt_fma")]
    build.define("FMA_OPT", None).flag("-mfma");

    #[cfg(all(feature = "fft_spqlios", not(feature = "fft_spqlios_avx512")))]
    build
        .file("./MOSFHET/src/fft/spqlios/spqlios-fft-fma.s")
        .file("./MOSFHET/src/fft/spqlios/spqlios-ifft-fma.s")
        .file("./MOSFHET/src/fft/spqlios/spqlios-fft-impl.c")
        .file("./MOSFHET/src/fft/spqlios/fft_processor_spqlios.c")
        .define("USE_SPQLIOS", None)
        .flag("-mfma")
        .flag("-msse2");

    #[cfg(feature = "fft_spqlios_avx512")]
    build
        .file("./MOSFHET/src/fft/spqlios/spqlios-fft-avx512.s")
        .file("./MOSFHET/src/fft/spqlios/spqlios-ifft-avx512.s")
        .file("./MOSFHET/src/fft/spqlios/spqlios-fft-impl-avx512.c")
        .file("./MOSFHET/src/fft/spqlios/fft_processor_spqlios.c")
        .define("USE_SPQLIOS", None)
        .define("AVX512_OPT", None)
        .flag("-mfma")
        .flag("-msse2")
        .flag("-mavx512dq")
        .flag("-mavx512f");

    #[cfg(feature = "rng_vaes")]
    build
        .file("./MOSFHET/src/trlwe_compressed_vaes.c")
        .define("USE_COMPRESSED_TRLWE", None)
        .define("USE_VAES", None)
        .define("VAES_OPT", None)
        .flag("-mvaes");

    #[cfg(feature = "rng_shake")]
    build
        .file("./MOSFHET/src/trlwe_compressed.c")
        .define("USE_COMPRESSED_TRLWE", None)
        .define("USE_SHAKE", None);

    #[cfg(feature = "rng_xoshiro")]
    build
        .file("./MOSFHET/src/trlwe_compressed.c")
        .define("USE_COMPRESSED_TRLWE", None);

    #[cfg(feature = "torus32")]
    build.define("TORUS32", None);

    if build.get_compiler().is_like_clang() {
        build.flag("-Wno-unused-command-line-argument");
    }

    build
        .flag("-Wno-sign-compare")
        .flag("-Wno-unused-result")
        .flag("-Wno-unused-variable")
        .flag("-Wno-unused-parameter")
        .static_flag(true)
        .compile("libmosfhet.a");

    println!("cargo:rustc-link-lib=static=mosfhet");

    #[cfg(feature = "bindgen")]
    bindgen::Builder::default()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .header("./MOSFHET/include/mosfhet.h")
        .clang_arg("-I./MOSFHET/include")
        .clang_arg("-D__AVX512FP16INTRIN_H")
        .clang_arg("-D__AVX512VLFP16INTRIN_H")
        .ctypes_prefix("::libc")
        .allowlist_type("Binary.*")
        .allowlist_type("Bootstrap_Key")
        .allowlist_type("Bootstrap_GA_Key")
        .allowlist_type("DFT_Polynomial")
        .allowlist_type("Generic_KS_Key")
        .allowlist_type("LUT_Packing_KS_Key")
        .allowlist_type("TLWE.*")
        .allowlist_type("TRGSW.*")
        .allowlist_type("TRLWE.*")
        .allowlist_function("blind_rotate")
        .allowlist_function(".*2torus")
        .allowlist_function("free_.*")
        .allowlist_function("functional_bootstrap_.*")
        .allowlist_function("generate_.*")
        .allowlist_function("init_fft")
        .allowlist_function("load_new_bootstrap_key.*")
        .allowlist_function("multivalue_bootstrap_.*")
        .allowlist_function("new_bootstrap_key.*")
        .allowlist_function("polynomial_.*")
        .allowlist_function("save_bootstrap_key.*")
        .allowlist_function("torus.*")
        .allowlist_function("tlwe_.*")
        .allowlist_function("trgsw_.*")
        .allowlist_function("trlwe_.*")
        .blocklist_type("FILE")
        .blocklist_type("__off_t")
        .blocklist_type("__off64_t")
        .blocklist_type("_IO_lock_t")
        .blocklist_type("_IO_FILE")
        .blocklist_type("_IO_codecvt")
        .blocklist_type("_IO_marker")
        .blocklist_type("_IO_wide_data")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
