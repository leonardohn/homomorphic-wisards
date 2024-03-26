#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]

use libc::FILE;

type __off_t = libc::c_long;
type __off64_t = libc::c_long;
type _IO_lock_t = libc::c_void;

#[cfg(not(feature = "bindgen"))]
include!("bindings.rs");

#[cfg(feature = "bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
