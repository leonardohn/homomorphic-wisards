use std::ops::{Index, IndexMut};

use crate::common::macros::*;

#[repr(transparent)]
pub struct BinaryPolynomial {
    ptr: mosfhet_sys::BinaryPolynomial,
}

impl BinaryPolynomial {
    pub(crate) unsafe fn new_uninit(upper_n: u32) -> Self {
        Self {
            ptr: mosfhet_sys::polynomial_new_binary_polynomial(upper_n as i32),
        }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*self.ptr).N as u32 }
    }

    pub fn as_slice(&self) -> &[mosfhet_sys::Binary] {
        unsafe {
            let len = (*self.ptr).N as usize;
            let ptr = (*self.ptr).coeffs as *const mosfhet_sys::Binary;
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [mosfhet_sys::Binary] {
        unsafe {
            let len = (*self.ptr).N as usize;
            let ptr = (*self.ptr).coeffs as *mut mosfhet_sys::Binary;
            std::slice::from_raw_parts_mut(ptr, len)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &mosfhet_sys::Binary> {
        self.as_slice().iter()
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut mosfhet_sys::Binary> {
        self.as_slice_mut().iter_mut()
    }

    pub fn from_fn<F>(upper_n: u32, f: F) -> Self
    where
        F: Fn(usize) -> mosfhet_sys::Binary,
    {
        let mut out = unsafe { Self::new_uninit(upper_n) };
        for i in 0..(upper_n as usize) {
            out[i] = f(i);
        }
        out
    }

    pub fn from_elem(upper_n: u32, elem: mosfhet_sys::Binary) -> Self {
        Self::from_fn(upper_n, |_| elem)
    }

    pub fn zeroed(upper_n: u32) -> Self {
        Self::from_elem(upper_n, 0i16)
    }

    pub fn naive_mul(&self, other: &Self) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            assert_eq!(upper_n, other.upper_n());
            let mut output = Self::new_uninit(upper_n);
            output.naive_mul_from(self, other);
            output
        }
    }

    pub fn naive_mul_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_naive_mul_binary(self.ptr, lhs.ptr, rhs.ptr)
        }
    }
}

impl_drop!(BinaryPolynomial => free_polynomial);
impl_ptrs!(BinaryPolynomial);

unsafe impl Send for BinaryPolynomial {}
unsafe impl Sync for BinaryPolynomial {}

impl Clone for BinaryPolynomial {
    fn clone(&self) -> Self {
        unsafe {
            let mut out = Self::new_uninit(self.upper_n());
            out.as_slice_mut().clone_from_slice(self.as_slice());
            out
        }
    }
}

impl Index<usize> for BinaryPolynomial {
    type Output = mosfhet_sys::Binary;
    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl IndexMut<usize> for BinaryPolynomial {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.as_slice_mut().index_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uninit_upper_n() {
        let upper_n = 16;
        let poly = unsafe { BinaryPolynomial::new_uninit(upper_n) };
        assert_eq!(poly.upper_n(), upper_n);
    }

    #[test]
    fn zeroed_clone_iter_mut_index() {
        let upper_n = 16;
        let mut poly = BinaryPolynomial::zeroed(upper_n);
        let clone = poly.clone();
        assert_eq!(poly.as_slice(), clone.as_slice());
        poly.iter_mut()
            .enumerate()
            .for_each(|(i, p)| *p = i as i16 + 1);
        assert_eq!(poly[0], 1);
        assert_eq!(poly[upper_n as usize - 1], upper_n as i16);
        poly.iter()
            .zip(clone.iter())
            .for_each(|(p, e)| assert_ne!(p, e));
    }

    #[test]
    fn naive_mul() {
        let upper_n = 16;
        let poly = BinaryPolynomial::from_fn(upper_n, |i| (i & 1) as i16);
        let expected = [-8, 0, -6, 0, -4, 0, -2, 0, 0, 0, 2, 0, 4, 0, 6, 0];
        assert_eq!(poly.naive_mul(&poly).as_slice(), expected.as_slice());
    }
}
