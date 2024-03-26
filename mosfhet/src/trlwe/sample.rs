use std::ops::{Index, IndexMut};

use crate::common::macros::*;
use crate::common::Torus;
use crate::poly::TorusPolynomial;
use crate::tlwe::TlweArray;
use crate::trgsw::TrgswDftArray;
use crate::trlwe::{TrlweDft, TrlweKey, TrlwePKSKey};

#[repr(transparent)]
pub struct Trlwe {
    ptr: mosfhet_sys::TRLWE,
}

impl Trlwe {
    pub(crate) unsafe fn new_uninit(k: u32, upper_n: u32) -> Self {
        Self {
            ptr: mosfhet_sys::trlwe_alloc_new_sample(k as i32, upper_n as i32),
        }
    }

    pub fn new(m: TorusPolynomial, key: &TrlweKey) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trlwe_new_sample(
                    m.as_ptr() as *mut _,
                    key.as_ptr() as *mut _,
                )
            },
        }
    }

    pub fn set(&mut self, m: TorusPolynomial, key: &TrlweKey) {
        unsafe {
            mosfhet_sys::trlwe_sample(
                self.ptr,
                m.as_ptr() as *mut _,
                key.as_ptr() as *mut _,
            )
        }
    }

    pub fn new_noiseless(m: TorusPolynomial, k: u32, upper_n: u32) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trlwe_new_noiseless_trivial_sample(
                    m.as_ptr() as *mut _,
                    k as i32,
                    upper_n as i32,
                )
            },
        }
    }

    pub fn set_noiseless(&mut self, m: TorusPolynomial) {
        unsafe {
            mosfhet_sys::trlwe_noiseless_trivial_sample(
                self.ptr,
                m.as_ptr() as *mut _,
            )
        }
    }

    pub fn zeroed(key: &TrlweKey) -> Self {
        let upper_n = key.upper_n();
        let m = TorusPolynomial::zeroed(upper_n);
        Self::new(m, key)
    }

    pub fn zeroed_noiseless(k: u32, upper_n: u32) -> Self {
        let m = TorusPolynomial::zeroed(upper_n);
        Self::new_noiseless(m, k, upper_n)
    }

    pub fn from_tlwe_array(
        array: &TlweArray,
        skip: usize,
        offset: usize,
        key: &TrlwePKSKey,
    ) -> Self {
        let k = key.out_k();
        let upper_n = key.out_upper_n();
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.set_from_tlwe_array(array, skip, offset, key);
        output
    }

    pub fn set_from_tlwe_array(
        &mut self,
        array: &TlweArray,
        skip: usize,
        offset: usize,
        key: &TrlwePKSKey,
    ) {
        let array_len = array.len();
        let required_size = skip + offset;
        let upper_n = self.upper_n() as usize;
        assert!(array_len >= required_size);
        assert!(offset <= upper_n);
        unsafe {
            let start = array.as_ptr() as *mut mosfhet_sys::TLWE;
            mosfhet_sys::trlwe_full_packing_keyswitch(
                self.ptr,
                start.add(skip),
                offset as u64,
                key.as_ptr() as *mut _,
            )
        }
    }

    pub fn k(&self) -> u32 {
        unsafe { (*self.ptr).k as u32 }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*(*(*self.ptr).a)).N as u32 }
    }

    pub fn add_assign(&mut self, rhs: &Self) {
        unsafe { mosfhet_sys::trlwe_addto(self.ptr, rhs.ptr) }
    }

    pub fn add_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::trlwe_add(self.ptr, lhs.ptr, rhs.ptr) }
    }

    pub fn add(&self, other: &Self) -> Self {
        let k = self.k();
        let upper_n = self.upper_n();
        assert_eq!(k, other.k());
        assert_eq!(upper_n, other.upper_n());
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.add_from(self, other);
        output
    }

    pub fn sub_assign(&mut self, other: &Self) {
        unsafe { mosfhet_sys::trlwe_subto(self.ptr, other.ptr) }
    }

    pub fn sub_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::trlwe_sub(self.ptr, lhs.ptr, rhs.ptr) }
    }

    pub fn sub(&self, other: &Self) -> Self {
        let k = self.k();
        let upper_n = self.upper_n();
        assert_eq!(k, other.k());
        assert_eq!(upper_n, other.upper_n());
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.sub_from(self, other);
        output
    }

    pub fn neg_from(&mut self, input: &Self) {
        unsafe { mosfhet_sys::trlwe_negate(self.ptr, input.ptr) }
    }

    pub fn neg(&self) -> Self {
        let k = self.k();
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.neg_from(self);
        output
    }

    pub fn mul_by_xai_from(&mut self, source: &Self, a: u32) {
        unsafe { mosfhet_sys::trlwe_mul_by_xai(self.ptr, source.ptr, a as i32) }
    }

    pub fn mul_by_xai(&self, a: u32) -> Self {
        let k = self.k();
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.mul_by_xai_from(self, a);
        output
    }

    pub fn mul_by_xai_pred_from(&mut self, source: &Self, a: u32) {
        unsafe {
            mosfhet_sys::trlwe_mul_by_xai_minus_1(
                self.ptr, source.ptr, a as i32,
            )
        }
    }

    pub fn mul_by_xai_pred(&self, a: u32) -> Self {
        let k = self.k();
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.mul_by_xai_pred_from(self, a);
        output
    }

    pub fn mul_by_xai_add_assign(&mut self, source: &Self, a: u32) {
        unsafe {
            mosfhet_sys::trlwe_mul_by_xai_addto(self.ptr, source.ptr, a as i32)
        }
    }

    pub fn blind_rotate(&mut self, a: &[Torus], s: &TrgswDftArray) {
        assert_eq!(a.len(), s.len());
        unsafe {
            mosfhet_sys::blind_rotate(
                self.ptr,
                a.as_ptr() as *mut _,
                s.as_ptr() as *mut _,
                s.len() as i32,
            )
        }
    }

    pub fn phase(&self, key: &TrlweKey) -> TorusPolynomial {
        unsafe {
            let upper_n = key.upper_n();
            let mut poly = TorusPolynomial::new_uninit(upper_n);
            let key_ptr = key.as_ptr() as *mut _;
            let poly_ptr = poly.as_ptr_mut() as *mut _;
            mosfhet_sys::trlwe_phase(poly_ptr, self.ptr, key_ptr);
            poly
        }
    }

    pub fn from_dft(sample: &TrlweDft) -> Self {
        unsafe {
            let k = sample.k();
            let upper_n = sample.upper_n();
            let mut output = Self::new_uninit(k, upper_n);
            output.set_from_dft(sample);
            output
        }
    }

    pub fn set_from_dft(&mut self, sample: &TrlweDft) {
        unsafe {
            mosfhet_sys::trlwe_from_DFT(self.ptr, sample.as_ptr() as *mut _)
        }
    }
}

impl_load!(Trlwe => trlwe_load_new_sample(k: u32, lower_n: u32));
impl_save!(Trlwe => trlwe_save_sample);
impl_drop!(Trlwe => free_trlwe);
impl_ptrs!(Trlwe);

unsafe impl Send for Trlwe {}
unsafe impl Sync for Trlwe {}

impl Clone for Trlwe {
    fn clone(&self) -> Self {
        let k = self.k();
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.clone_from(self);
        output
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe { mosfhet_sys::trlwe_copy(self.ptr, source.ptr) }
    }
}

pub struct TrlweArray {
    len: usize,
    ptr: *mut mosfhet_sys::TRLWE,
}

impl TrlweArray {
    pub(crate) unsafe fn new_uninit(len: usize, k: u32, upper_n: u32) -> Self {
        let ptr = mosfhet_sys::trlwe_alloc_new_sample_array(
            len as i32,
            k as i32,
            upper_n as i32,
        );
        Self { len, ptr }
    }

    pub fn from_fn<F>(len: usize, key: &TrlweKey, f: F) -> Self
    where
        F: Fn(usize) -> TorusPolynomial,
    {
        let mut output =
            unsafe { Self::new_uninit(len, key.k(), key.upper_n()) };
        output
            .as_slice_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| p.set(f(i), key));
        output
    }

    pub fn from_fn_noiseless<F>(len: usize, k: u32, upper_n: u32, f: F) -> Self
    where
        F: Fn(usize) -> TorusPolynomial,
    {
        let mut output = unsafe { Self::new_uninit(len, k, upper_n) };
        output
            .as_slice_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| p.set_noiseless(f(i)));
        output
    }

    pub fn from_elem(
        len: usize,
        key: &TrlweKey,
        elem: TorusPolynomial,
    ) -> Self {
        Self::from_fn(len, key, |_| elem.clone())
    }

    pub fn from_elem_noiseless(
        len: usize,
        k: u32,
        upper_n: u32,
        elem: TorusPolynomial,
    ) -> Self {
        Self::from_fn_noiseless(len, k, upper_n, |_| elem.clone())
    }

    pub fn zeroed(len: usize, key: &TrlweKey) -> Self {
        let upper_n = key.upper_n();
        let elem = TorusPolynomial::zeroed(upper_n);
        Self::from_elem(len, key, elem)
    }

    pub fn zeroed_noiseless(len: usize, k: u32, upper_n: u32) -> Self {
        let elem = TorusPolynomial::zeroed(upper_n);
        Self::from_elem_noiseless(len, k, upper_n, elem)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn k(&self) -> u32 {
        unsafe { (*(*self.ptr)).k as u32 }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*(*(*self.ptr)).b).N as u32 }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Trlwe> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Trlwe> {
        self.as_slice_mut().iter_mut()
    }
}

impl_load_array!(TrlweArray => trlwe_load_sample(k: u32, lower_n: u32));
impl_save_array!(TrlweArray => trlwe_save_sample);
impl_drop_array!(TrlweArray => free_trlwe_array);
impl_slice_array!(TrlweArray);
impl_ptrs!(TrlweArray);

unsafe impl Send for TrlweArray {}
unsafe impl Sync for TrlweArray {}

impl Clone for TrlweArray {
    fn clone(&self) -> Self {
        unsafe {
            let k = self.k();
            let len = self.len();
            let upper_n = self.upper_n();
            let mut output = Self::new_uninit(len, k, upper_n);
            output.clone_from(self);
            output
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.as_slice_mut().clone_from_slice(source.as_slice());
    }
}

impl Index<usize> for TrlweArray {
    type Output = Trlwe;

    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl IndexMut<usize> for TrlweArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.as_slice_mut().index_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn full_packing_key_switch() {
        // TFHE parameters
        let upper_n = 1024usize;
        let sigma = 2.9802322387695312e-8f64;
        let k = 1;
        let t = 8;
        let base_bit = 4;

        // Set output precision
        let out_prec = 6;

        // Generate new keys
        let trlwe_key = TrlweKey::new(upper_n as u32, k, sigma);
        let tlwe_key = TlweKey::from_trlwe_key(&trlwe_key);
        let trlwe_pks_key =
            TrlwePKSKey::new(&tlwe_key, &trlwe_key, t, base_bit);

        // Encrypt a TLWE array
        let tlwe_array = TlweArray::from_fn(upper_n + 6, &tlwe_key, |i| {
            Torus::from_unsigned(
                i as RawTorus & ((1 << out_prec) - 1),
                out_prec,
            )
        });

        // Perform the packing key switch
        let trlwe =
            Trlwe::from_tlwe_array(&tlwe_array, 3, upper_n, &trlwe_pks_key);
        let poly = trlwe.phase(&trlwe_key);

        // Check if the values match
        for (i, v) in poly.iter().enumerate() {
            assert_eq!(
                v.into_unsigned(out_prec),
                (i as RawTorus + 3) & ((1 << out_prec) - 1)
            );
        }
    }
}
