use std::ops::{Index, IndexMut};

use crate::common::macros::*;
use crate::common::Torus;
use crate::poly::DftPolynomial;
use crate::trgsw::Trgsw;
use crate::trlwe::{Trlwe, TrlweDft};

use super::TrgswKey;

#[repr(transparent)]
pub struct TrgswDft {
    ptr: mosfhet_sys::TRGSW_DFT,
}

impl TrgswDft {
    pub(crate) unsafe fn new_uninit(
        l: u32,
        bg_bit: u32,
        k: u32,
        upper_n: u32,
    ) -> Self {
        Self {
            ptr: mosfhet_sys::trgsw_alloc_new_DFT_sample(
                l as i32,
                bg_bit as i32,
                k as i32,
                upper_n as i32,
            ),
        }
    }

    pub fn from_trgsw(sample: &Trgsw) -> Self {
        let l = sample.l();
        let bg_bit = sample.bg_bit();
        let k = sample.k();
        let upper_n = sample.upper_n();
        let mut output = unsafe { Self::new_uninit(l, bg_bit, k, upper_n) };
        output.set_from_trgsw(sample);
        output
    }

    pub fn set_from_trgsw(&mut self, sample: &Trgsw) {
        unsafe {
            mosfhet_sys::trgsw_to_DFT(self.ptr, sample.as_ptr() as *mut _)
        }
    }

    pub fn l(&self) -> u32 {
        unsafe { (*self.ptr).l as u32 }
    }

    pub fn bg_bit(&self) -> u32 {
        unsafe { (*self.ptr).Bg_bit as u32 }
    }

    pub fn k(&self) -> u32 {
        unsafe { (*(*(*self.ptr).samples)).k as u32 }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*(*(*(*self.ptr).samples)).b).N as u32 }
    }

    pub fn add_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::trgsw_DFT_add(self.ptr, lhs.ptr, rhs.ptr) }
    }

    pub fn add(&self, other: &Self) -> Self {
        let l = self.l();
        let bg_bit = self.bg_bit();
        let k = self.k();
        let upper_n = self.upper_n();
        assert_eq!(l, other.l());
        assert_eq!(bg_bit, other.bg_bit());
        assert_eq!(k, other.k());
        assert_eq!(upper_n, other.upper_n());
        let mut output = unsafe { Self::new_uninit(l, bg_bit, k, upper_n) };
        output.add_from(self, other);
        output
    }

    pub fn sub_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::trgsw_DFT_sub(self.ptr, lhs.ptr, rhs.ptr) }
    }

    pub fn sub(&self, other: &Self) -> Self {
        let l = self.l();
        let bg_bit = self.bg_bit();
        let k = self.k();
        let upper_n = self.upper_n();
        assert_eq!(l, other.l());
        assert_eq!(bg_bit, other.bg_bit());
        assert_eq!(k, other.k());
        assert_eq!(upper_n, other.upper_n());
        let mut output = unsafe { Self::new_uninit(l, bg_bit, k, upper_n) };
        output.sub_from(self, other);
        output
    }

    pub fn mul_add_assign_poly(&mut self, other: &Self, poly: &DftPolynomial) {
        unsafe {
            mosfhet_sys::trgsw_DFT_mul_addto_by_polynomial(
                self.ptr,
                other.ptr,
                poly.as_ptr() as *mut _,
            )
        }
    }

    pub fn cmux(&self, in0: &mut Trlwe, in1: &mut Trlwe) {
        in1.sub_assign(in0);
        in1.set_from_dft(&TrlweDft::mul_trlwe_dft(in1, self));
        in0.add_assign(in1);
    }

    pub fn cmux_vectored(&self, in_: &mut [Trlwe], stride: usize) {
        let chunk_size = stride + stride;
        let spare_items = in_.len() % chunk_size;

        for chunk in in_.chunks_exact_mut(chunk_size) {
            let (lower, upper) = chunk.split_at_mut(stride);
            self.cmux(&mut lower[0], &mut upper[0]);
        }

        if spare_items > stride {
            let offset = in_.len() - spare_items;
            let chunk = &mut in_[offset..];
            let (lower, upper) = chunk.split_at_mut(stride);
            self.cmux(&mut lower[0], &mut upper[0]);
        }
    }

    pub fn cdemux(&self, in0: &mut Trlwe, in1: &mut Trlwe) {
        in1.add_assign(in0);
        in1.set_from_dft(&TrlweDft::mul_trlwe_dft(in1, self));
        in0.sub_assign(in1);
    }

    pub fn cdemux_vectored(&self, in_: &mut [Trlwe], stride: usize) {
        let chunk_size = stride + stride;
        let spare_items = in_.len() % chunk_size;

        for chunk in in_.chunks_exact_mut(chunk_size) {
            let (lower, upper) = chunk.split_at_mut(stride);
            self.cdemux(&mut lower[0], &mut upper[0]);
        }

        if spare_items > stride {
            let offset = in_.len() - spare_items;
            let chunk = &mut in_[offset..];
            let (lower, upper) = chunk.split_at_mut(stride);
            self.cdemux(&mut lower[0], &mut upper[0]);
        }
    }

    pub fn cswap(&self, in0: &mut Trlwe, in1: &mut Trlwe) {
        let mut delta = Trlwe::sub(in0, in1);
        delta.set_from_dft(&TrlweDft::mul_trlwe_dft(&delta, self));
        in0.sub_assign(&delta);
        in1.add_assign(&delta);
    }

    pub fn cswap_vectored(
        &self,
        in_: &mut [Trlwe],
        offset: usize,
        stride: usize,
    ) {
        assert!(offset < stride);
        let chunk_size = stride + stride;
        let spare_left = offset;
        let spare_right = (in_.len() - spare_left) % chunk_size;
        let offset_slice = &mut in_[offset..];

        for chunk in offset_slice.chunks_exact_mut(chunk_size) {
            let (lower, upper) = chunk.split_at_mut(stride);
            self.cswap(&mut lower[0], &mut upper[0]);
        }

        if spare_right > stride {
            let offset = in_.len() - spare_right;
            let chunk = &mut in_[offset..];
            let (lower, upper) = chunk.split_at_mut(stride);
            self.cswap(&mut lower[0], &mut upper[0]);
        } else if spare_left + spare_right > stride {
            let offset = in_.len() - spare_right;
            let (lower, upper) = in_.split_at_mut(offset);
            let wrap_idx = stride - spare_right;
            self.cswap(&mut upper[0], &mut lower[wrap_idx]);
        }
    }
}

impl_load!(
    TrgswDft => trgsw_load_new_DFT_sample(
        l: u32,
        bg_bit: u32,
        k: u32,
        lower_n: u32,
    )
);
impl_save!(TrgswDft => trgsw_save_DFT_sample);
impl_drop!(TrgswDft => free_trgsw);
impl_ptrs!(TrgswDft);

unsafe impl Send for TrgswDft {}
unsafe impl Sync for TrgswDft {}

impl Clone for TrgswDft {
    fn clone(&self) -> Self {
        let l = self.l();
        let bg_bit = self.bg_bit();
        let k = self.k();
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(l, bg_bit, k, upper_n) };
        output.clone_from(self);
        output
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe { mosfhet_sys::trgsw_DFT_copy(self.ptr, source.ptr) }
    }
}

pub struct TrgswDftArray {
    len: usize,
    ptr: *mut mosfhet_sys::TRGSW_DFT,
}

impl TrgswDftArray {
    pub(crate) unsafe fn new_uninit(
        len: usize,
        l: u32,
        bg_bit: u32,
        k: u32,
        upper_n: u32,
    ) -> Self {
        let ptr = mosfhet_sys::trgsw_alloc_new_DFT_sample_array(
            len as i32,
            l as i32,
            bg_bit as i32,
            k as i32,
            upper_n as i32,
        );
        Self { len, ptr }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn l(&self) -> u32 {
        unsafe { (*(*self.ptr)).l as u32 }
    }

    pub fn bg_bit(&self) -> u32 {
        unsafe { (*(*self.ptr)).Bg_bit as u32 }
    }

    pub fn k(&self) -> u32 {
        unsafe { (*(*(*(*self.ptr)).samples)).k as u32 }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*(*(*(*(*self.ptr)).samples)).b).N as u32 }
    }

    pub fn from_fn<F>(len: usize, key: &TrgswKey, f: F) -> Self
    where
        F: Fn(usize) -> (Torus, u32),
    {
        let mut output = unsafe {
            Self::new_uninit(len, key.l(), key.bg_bit(), key.k(), key.upper_n())
        };
        output
            .as_slice_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                let (m, e) = f(i);
                p.set_from_trgsw(&Trgsw::new(m, e, key));
            });
        output
    }

    pub fn from_elem(len: usize, key: &TrgswKey, m: Torus, e: u32) -> Self {
        Self::from_fn(len, key, |_| (m, e))
    }

    pub fn iter(&self) -> impl Iterator<Item = &TrgswDft> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TrgswDft> {
        self.as_slice_mut().iter_mut()
    }
}

impl_load_array!(
    TrgswDftArray => trgsw_load_DFT_sample(
        l: u32,
        bg_bit: u32,
        k: u32,
        lower_n: u32,
    )
);
impl_save_array!(TrgswDftArray => trgsw_save_DFT_sample);
impl_drop_array!(TrgswDftArray => free_trgsw_array);
impl_slice_array!(TrgswDftArray);
impl_ptrs!(TrgswDftArray);

unsafe impl Send for TrgswDftArray {}
unsafe impl Sync for TrgswDftArray {}

impl Clone for TrgswDftArray {
    fn clone(&self) -> Self {
        unsafe {
            let mut output = Self::new_uninit(
                self.len(),
                self.l(),
                self.bg_bit(),
                self.k(),
                self.upper_n(),
            );
            output.clone_from(self);
            output
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.as_slice_mut().clone_from_slice(source.as_slice());
    }
}

impl Index<usize> for TrgswDftArray {
    type Output = TrgswDft;

    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl IndexMut<usize> for TrgswDftArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.as_slice_mut().index_mut(index)
    }
}
