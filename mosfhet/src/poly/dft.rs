use std::ops::{Index, IndexMut};

use crate::common::macros::*;
use crate::poly::TorusPolynomial;

#[repr(transparent)]
pub struct DftPolynomial {
    ptr: mosfhet_sys::DFT_Polynomial,
}

impl DftPolynomial {
    pub(crate) unsafe fn new_uninit(upper_n: u32) -> Self {
        Self {
            ptr: mosfhet_sys::polynomial_new_DFT_polynomial(upper_n as i32),
        }
    }

    pub fn from_torus(src: &TorusPolynomial) -> Self {
        unsafe {
            let mut out = Self::new_uninit(src.upper_n());
            out.set_from_torus(src);
            out
        }
    }

    pub fn set_from_torus(&mut self, src: &TorusPolynomial) {
        unsafe {
            mosfhet_sys::polynomial_torus_to_DFT(
                self.ptr,
                src.as_ptr() as *mut _,
            );
        }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*self.ptr).N as u32 }
    }

    pub fn as_slice(&self) -> &[f64] {
        unsafe {
            let len = (*self.ptr).N as usize;
            let ptr = (*self.ptr).coeffs as *const f64;
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [f64] {
        unsafe {
            let len = (*self.ptr).N as usize;
            let ptr = (*self.ptr).coeffs;
            std::slice::from_raw_parts_mut(ptr, len)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f64> {
        self.as_slice_mut().iter_mut()
    }

    pub fn from_fn<F>(upper_n: u32, f: F) -> Self
    where
        F: Fn(usize) -> f64,
    {
        let mut output = unsafe { Self::new_uninit(upper_n) };
        for i in 0..(upper_n as usize) {
            output[i] = f(i);
        }
        output
    }

    pub fn from_elem(upper_n: u32, elem: f64) -> Self {
        Self::from_fn(upper_n, |_| elem)
    }

    pub fn zeroed(upper_n: u32) -> Self {
        Self::from_elem(upper_n, 0.0)
    }

    pub fn add(&self, other: &Self) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            assert_eq!(upper_n, other.upper_n());
            let mut output = Self::new_uninit(upper_n);
            output.add_from(self, other);
            output
        }
    }

    pub fn add_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_add_DFT_polynomials(
                self.ptr, lhs.ptr, rhs.ptr,
            )
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            assert_eq!(upper_n, other.upper_n());
            let mut output = Self::new_uninit(upper_n);
            output.sub_from(self, other);
            output
        }
    }

    pub fn sub_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_sub_DFT_polynomials(
                self.ptr, lhs.ptr, rhs.ptr,
            )
        }
    }

    pub fn mul(&self, other: &Self) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            assert_eq!(upper_n, other.upper_n());
            let mut output = Self::new_uninit(upper_n);
            output.mul_from(self, other);
            output
        }
    }

    pub fn mul_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::polynomial_mul_DFT(self.ptr, lhs.ptr, rhs.ptr) }
    }

    pub fn mul_add_assign(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_mul_addto_DFT(self.ptr, lhs.ptr, rhs.ptr)
        }
    }
}

impl_drop!(DftPolynomial => free_DFT_polynomial);
impl_ptrs!(DftPolynomial);

unsafe impl Send for DftPolynomial {}
unsafe impl Sync for DftPolynomial {}

impl Clone for DftPolynomial {
    fn clone(&self) -> Self {
        unsafe {
            let mut output = Self::new_uninit(self.upper_n());
            output.clone_from(self);
            output
        }
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            mosfhet_sys::polynomial_copy_DFT_polynomial(self.ptr, source.ptr)
        }
    }
}

impl Index<usize> for DftPolynomial {
    type Output = f64;
    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl IndexMut<usize> for DftPolynomial {
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
        let poly = unsafe { DftPolynomial::new_uninit(upper_n) };
        assert_eq!(poly.upper_n(), upper_n);
    }

    #[test]
    fn from_torus() {
        use crate::poly::TorusPolynomial;
        let upper_n = 16;
        let poly_torus = TorusPolynomial::zeroed(upper_n);
        let poly_dft = DftPolynomial::from_torus(&poly_torus);
        let expected = DftPolynomial::zeroed(upper_n);
        assert_eq!(poly_dft.as_slice(), expected.as_slice());
    }

    #[test]
    fn zeroed_clone_iter_mut_index() {
        let upper_n = 16;
        let mut poly = DftPolynomial::zeroed(upper_n);
        let clone = poly.clone();
        assert_eq!(poly.as_slice(), clone.as_slice());
        poly.iter_mut()
            .enumerate()
            .for_each(|(i, p)| *p = i as f64 + 1.0);
        assert_eq!(poly[0], 1.0);
        assert_eq!(poly[upper_n as usize - 1], upper_n as f64);
        poly.iter()
            .zip(clone.iter())
            .for_each(|(p, e)| assert_ne!(p, e));
    }

    #[test]
    fn from_elem_add_sub_mul_muladd() {
        let upper_n = 16;
        let a = DftPolynomial::from_elem(upper_n, 1.0);
        let b = DftPolynomial::from_elem(upper_n, 2.0);
        let c = a.add(&a);
        assert_eq!(c.as_slice(), b.as_slice());
        let c = b.sub(&a);
        assert_eq!(c.as_slice(), a.as_slice());
        let c = a.mul(&b);
        let d = DftPolynomial::from_fn(upper_n, |i| {
            if i >= (upper_n as usize >> 1) {
                4.0
            } else {
                0.0
            }
        });
        assert_eq!(c.as_slice(), d.as_slice());
        let mut d = DftPolynomial::zeroed(upper_n);
        d.mul_add_assign(&a, &b);
        assert_eq!(c.as_slice(), d.as_slice());
    }
}
