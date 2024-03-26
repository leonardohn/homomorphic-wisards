use std::ops::{Index, IndexMut};

use crate::common::macros::*;
use crate::common::Torus;
use crate::poly::{BinaryPolynomial, DftPolynomial};

#[repr(transparent)]
pub struct TorusPolynomial {
    ptr: mosfhet_sys::TorusPolynomial,
}

impl TorusPolynomial {
    pub(crate) unsafe fn new_uninit(upper_n: u32) -> Self {
        Self {
            ptr: mosfhet_sys::polynomial_new_torus_polynomial(upper_n as i32),
        }
    }

    pub fn from_dft(src: &DftPolynomial) -> Self {
        unsafe {
            let mut out = Self::new_uninit(src.upper_n());
            out.set_from_dft(src);
            out
        }
    }

    pub fn set_from_dft(&mut self, src: &DftPolynomial) {
        unsafe {
            mosfhet_sys::polynomial_DFT_to_torus(
                self.ptr,
                src.as_ptr() as *mut _,
            );
        }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*self.ptr).N as u32 }
    }

    pub fn as_slice(&self) -> &[Torus] {
        unsafe {
            let len = (*self.ptr).N as usize;
            let ptr = (*self.ptr).coeffs as *const Torus;
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [Torus] {
        unsafe {
            let len = (*self.ptr).N as usize;
            let ptr = (*self.ptr).coeffs as *mut Torus;
            std::slice::from_raw_parts_mut(ptr, len)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Torus> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Torus> {
        self.as_slice_mut().iter_mut()
    }

    pub fn from_fn<F>(upper_n: u32, f: F) -> Self
    where
        F: Fn(usize) -> Torus,
    {
        let mut output = unsafe { Self::new_uninit(upper_n) };
        for i in 0..(upper_n as usize) {
            output[i] = f(i);
        }
        output
    }

    pub fn from_elem(upper_n: u32, elem: Torus) -> Self {
        Self::from_fn(upper_n, |_| elem)
    }

    pub fn zeroed(upper_n: u32) -> Self {
        Self::from_elem(upper_n, Torus::MIN)
    }

    pub fn add(&self, other: &Self) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            assert_eq!(upper_n, other.upper_n());
            let mut result = Self::new_uninit(upper_n);
            result.add_from(self, other);
            result
        }
    }

    pub fn add_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_add_torus_polynomials(
                self.ptr, lhs.ptr, rhs.ptr,
            )
        }
    }

    pub fn add_assign(&mut self, other: &Self) {
        unsafe {
            mosfhet_sys::polynomial_addto_torus_polynomial(self.ptr, other.ptr)
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            assert_eq!(upper_n, other.upper_n());
            let mut result = Self::new_uninit(upper_n);
            result.sub_from(self, other);
            result
        }
    }

    pub fn sub_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_sub_torus_polynomials(
                self.ptr, lhs.ptr, rhs.ptr,
            )
        }
    }

    pub fn sub_assign(&mut self, other: &Self) {
        unsafe {
            mosfhet_sys::polynomial_subto_torus_polynomial(self.ptr, other.ptr)
        }
    }

    pub fn mul(&self, other: &Self) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            assert_eq!(upper_n, other.upper_n());
            let mut result = Self::new_uninit(upper_n);
            result.mul_from(self, other);
            result
        }
    }

    pub fn mul_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::polynomial_mul_torus(self.ptr, lhs.ptr, rhs.ptr) }
    }

    pub fn mul_add_assign(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_mul_addto_torus(self.ptr, lhs.ptr, rhs.ptr)
        }
    }

    pub fn mul_by_xai(&self, a: u32) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            let mut result = Self::new_uninit(upper_n);
            result.mul_by_xai_from(self, a);
            result
        }
    }

    pub fn mul_by_xai_from(&mut self, source: &Self, a: u32) {
        unsafe {
            mosfhet_sys::torus_polynomial_mul_by_xai(
                self.ptr, source.ptr, a as i32,
            )
        }
    }

    pub fn mul_by_xai_pred(&self, a: u32) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            let mut result = Self::new_uninit(upper_n);
            result.mul_by_xai_pred_from(self, a);
            result
        }
    }

    pub fn mul_by_xai_pred_from(&mut self, source: &Self, a: u32) {
        unsafe {
            mosfhet_sys::torus_polynomial_mul_by_xai_minus_1(
                self.ptr, source.ptr, a as i32,
            )
        }
    }

    pub fn mul_by_xai_add_assign(&mut self, other: &Self, a: u32) {
        unsafe {
            mosfhet_sys::torus_polynomial_mul_by_xai_addto(
                self.ptr, other.ptr, a as i32,
            )
        }
    }

    pub fn mul_scale(
        &mut self,
        lhs: &Self,
        rhs: &Self,
        bit_size: u32,
        scale_bit: u32,
    ) {
        unsafe {
            mosfhet_sys::polynomial_full_mul_with_scale(
                self.ptr,
                lhs.ptr,
                rhs.ptr,
                bit_size as i32,
                scale_bit as i32,
            )
        }
    }

    pub fn naive_mul(&self, other: &Self) -> Self {
        unsafe {
            let upper_n = self.upper_n();
            assert_eq!(upper_n, other.upper_n());
            let mut out = Self::new_uninit(upper_n);
            out.naive_mul_from(self, other);
            out
        }
    }

    pub fn naive_mul_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_naive_mul_torus(self.ptr, lhs.ptr, rhs.ptr)
        }
    }

    pub fn naive_mul_add_assign(&mut self, lhs: &Self, rhs: &Self) {
        unsafe {
            mosfhet_sys::polynomial_naive_mul_addto_torus(
                self.ptr, lhs.ptr, rhs.ptr,
            )
        }
    }

    pub fn naive_mul_add_assign_binary(
        &mut self,
        lhs: &Self,
        rhs: &BinaryPolynomial,
    ) {
        unsafe {
            mosfhet_sys::polynomial_naive_mul_addto_torus_binary(
                self.ptr,
                lhs.ptr,
                rhs.as_ptr() as *mut _,
            )
        }
    }

    pub fn neg_from(&mut self, source: &Self) {
        unsafe {
            mosfhet_sys::polynomial_negate_torus_polynomial(
                self.ptr, source.ptr,
            )
        }
    }

    pub fn neg(&self) -> Self {
        let upper_n = self.upper_n();
        let mut out = unsafe { Self::new_uninit(upper_n) };
        out.neg_from(self);
        out
    }

    pub fn decompose_index_from(
        &mut self,
        source: &Self,
        bg_bit: u32,
        l: u32,
        i: u32,
    ) {
        unsafe {
            mosfhet_sys::polynomial_decompose_i(
                self.ptr,
                source.ptr,
                bg_bit as i32,
                l as i32,
                i as i32,
            )
        }
    }

    pub fn decompose_index(&self, bg_bit: u32, l: u32, index: u32) -> Self {
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(upper_n) };
        output.decompose_index_from(self, bg_bit, l, index);
        output
    }

    pub fn permute_from(&mut self, source: &Self, gen: u64) {
        unsafe { mosfhet_sys::polynomial_permute(self.ptr, source.ptr, gen) }
    }

    pub fn permute(&self, gen: u64) -> Self {
        let upper_n = self.upper_n();
        let mut out = unsafe { Self::new_uninit(upper_n) };
        out.permute_from(self, gen);
        out
    }

    pub fn scale_from(&mut self, source: &Self, log_scale: usize) {
        unsafe {
            mosfhet_sys::polynomial_torus_scale(
                self.ptr,
                source.ptr,
                log_scale as i32,
            )
        }
    }

    pub fn scale(&self, log_scale: usize) -> Self {
        let upper_n = self.upper_n();
        let mut out = unsafe { Self::new_uninit(upper_n) };
        out.scale_from(self, log_scale);
        out
    }
}

impl_drop!(TorusPolynomial => free_polynomial);
impl_ptrs!(TorusPolynomial);

unsafe impl Send for TorusPolynomial {}
unsafe impl Sync for TorusPolynomial {}

impl Clone for TorusPolynomial {
    fn clone(&self) -> Self {
        unsafe {
            let mut out = Self::new_uninit(self.upper_n());
            out.clone_from(self);
            out
        }
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            mosfhet_sys::polynomial_copy_torus_polynomial(self.ptr, source.ptr)
        }
    }
}

impl Index<usize> for TorusPolynomial {
    type Output = Torus;
    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl IndexMut<usize> for TorusPolynomial {
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
        let poly = unsafe { TorusPolynomial::new_uninit(upper_n) };
        assert_eq!(poly.upper_n(), upper_n);
    }

    #[test]
    fn from_dft() {
        use crate::poly::DftPolynomial;
        let upper_n = 16;
        let poly_dft = DftPolynomial::zeroed(upper_n);
        let poly_torus = TorusPolynomial::from_dft(&poly_dft);
        let expected = TorusPolynomial::from_elem(upper_n, Torus::from_raw(2));
        assert_eq!(poly_torus.as_slice(), expected.as_slice());
    }

    #[test]
    fn zeroed_clone_iter_mut_index() {
        use crate::common::RawTorus;
        let upper_n = 16;
        let mut poly = TorusPolynomial::zeroed(upper_n);
        let clone = poly.clone();
        assert_eq!(poly.as_slice(), clone.as_slice());
        poly.iter_mut()
            .enumerate()
            .for_each(|(i, p)| *p = Torus::from_raw(i as RawTorus + 1));
        assert_eq!(poly[0], Torus::from_raw(1));
        assert_eq!(
            poly[upper_n as usize - 1],
            Torus::from_raw(upper_n as RawTorus)
        );
        poly.iter()
            .zip(clone.iter())
            .for_each(|(p, e)| assert_ne!(p, e));
    }

    #[test]
    fn from_elem_add_sub_mul_muladd_assign() {
        let upper_n = 8;
        let a = TorusPolynomial::from_elem(upper_n, Torus::from_unsigned(1, 8));
        let b = TorusPolynomial::from_elem(upper_n, Torus::from_unsigned(2, 8));
        let c = a.add(&a);
        assert_eq!(c.as_slice(), b.as_slice());
        let c = b.sub(&a);
        assert_eq!(c.as_slice(), a.as_slice());
        let mut c = TorusPolynomial::zeroed(upper_n);
        c.add_assign(&b);
        c.sub_assign(&a);
        assert_eq!(c.as_slice(), a.as_slice());
        let c = TorusPolynomial::zeroed(upper_n);
        let d = c.mul(&c);
        let e = [Torus::from_raw(2); 8];
        assert_eq!(d.as_slice(), e.as_slice());
        let mut d = TorusPolynomial::zeroed(upper_n);
        d.mul_add_assign(&c, &c);
        let e = [Torus::from_raw(2); 8];
        assert_eq!(d.as_slice(), e.as_slice());
    }

    #[test]
    fn mul_by_xai_pred_add_assign() {
        let upper_n = 8;
        let poly_a =
            TorusPolynomial::from_elem(upper_n, Torus::from_unsigned(1, 4));
        let expected = [
            Torus::from_unsigned(15, 4),
            Torus::from_unsigned(1, 4),
            Torus::from_unsigned(1, 4),
            Torus::from_unsigned(1, 4),
            Torus::from_unsigned(1, 4),
            Torus::from_unsigned(1, 4),
            Torus::from_unsigned(1, 4),
            Torus::from_unsigned(1, 4),
        ];
        let poly_b = poly_a.mul_by_xai(1);
        assert_eq!(poly_b.as_slice(), expected.as_slice());
        let mut poly_b = TorusPolynomial::zeroed(upper_n);
        poly_b.mul_by_xai_add_assign(&poly_a, 1);
        assert_eq!(poly_b.as_slice(), expected.as_slice());
        let expected = [
            Torus::from_unsigned(14, 4),
            Torus::from_unsigned(0, 4),
            Torus::from_unsigned(0, 4),
            Torus::from_unsigned(0, 4),
            Torus::from_unsigned(0, 4),
            Torus::from_unsigned(0, 4),
            Torus::from_unsigned(0, 4),
            Torus::from_unsigned(0, 4),
        ];
        let poly_b = poly_a.mul_by_xai_pred(1);
        assert_eq!(poly_b.as_slice(), expected.as_slice());
    }
}
