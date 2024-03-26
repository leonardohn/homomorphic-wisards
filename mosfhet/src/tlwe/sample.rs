use crate::common::macros::*;
use crate::common::Torus;
use crate::tlwe::{TlweKSKey, TlweKey};
use crate::trlwe::Trlwe;

use std::ops::{Index, IndexMut};

#[repr(transparent)]
pub struct Tlwe {
    ptr: mosfhet_sys::TLWE,
}

impl Tlwe {
    pub(crate) unsafe fn new_uninit(lower_n: u32) -> Self {
        let ptr = mosfhet_sys::tlwe_alloc_sample(lower_n as i32);
        Self { ptr }
    }

    pub fn new(m: Torus, key: &TlweKey) -> Self {
        let ptr = unsafe {
            mosfhet_sys::tlwe_new_sample(m.0, key.as_ptr() as *mut _)
        };
        Self { ptr }
    }

    pub fn set(&mut self, m: Torus, key: &TlweKey) {
        unsafe {
            mosfhet_sys::tlwe_sample(self.ptr, m.0, key.as_ptr() as *mut _)
        }
    }

    pub fn new_noiseless(m: Torus, lower_n: u32) -> Self {
        let ptr = unsafe {
            mosfhet_sys::tlwe_new_noiseless_trivial_sample(m.0, lower_n as i32)
        };
        Self { ptr }
    }

    pub fn set_noiseless(&mut self, m: Torus) {
        unsafe { mosfhet_sys::tlwe_noiseless_trivial_sample(self.ptr, m.0) }
    }

    pub fn zero(key: &TlweKey) -> Self {
        Self::new(Torus::MIN, key)
    }

    pub fn zero_noiseless(lower_n: u32) -> Self {
        Self::new_noiseless(Torus::MIN, lower_n)
    }

    pub fn from_trlwe(sample: &Trlwe, index: usize) -> Tlwe {
        let k = sample.k();
        let upper_n = sample.upper_n();
        let mut output = unsafe { Tlwe::new_uninit(k * upper_n) };
        output.set_from_trlwe(sample, index);
        output
    }

    pub fn set_from_trlwe(&mut self, sample: &Trlwe, index: usize) {
        unsafe {
            mosfhet_sys::trlwe_extract_tlwe(
                self.ptr,
                sample.as_ptr() as *mut _,
                index as i32,
            )
        }
    }

    pub fn lower_n(&self) -> u32 {
        unsafe { (*self.ptr).n as u32 }
    }

    pub fn add_assign(&mut self, rhs: &Self) {
        unsafe { mosfhet_sys::tlwe_addto(self.ptr, rhs.ptr) }
    }

    pub fn add_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::tlwe_add(self.ptr, lhs.ptr, rhs.ptr) }
    }

    pub fn add(&self, other: &Self) -> Self {
        let lower_n = self.lower_n();
        assert_eq!(lower_n, other.lower_n());
        let mut output = unsafe { Self::new_uninit(lower_n) };
        output.add_from(self, other);
        output
    }

    pub fn sub_assign(&mut self, other: &Self) {
        unsafe { mosfhet_sys::tlwe_subto(self.ptr, other.ptr) }
    }

    pub fn sub_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::tlwe_sub(self.ptr, lhs.ptr, rhs.ptr) }
    }

    pub fn sub(&self, other: &Self) -> Self {
        let lower_n = self.lower_n();
        assert_eq!(lower_n, other.lower_n());
        let mut output = unsafe { Self::new_uninit(lower_n) };
        output.sub_from(self, other);
        output
    }

    pub fn neg_from(&mut self, input: &Self) {
        unsafe { mosfhet_sys::tlwe_negate(self.ptr, input.ptr) }
    }

    pub fn neg(&self) -> Self {
        let lower_n = self.lower_n();
        let mut output = unsafe { Self::new_uninit(lower_n) };
        output.neg_from(self);
        output
    }

    pub fn phase(&self, key: &TlweKey) -> Torus {
        unsafe {
            Torus::from_raw(mosfhet_sys::tlwe_phase(
                self.ptr,
                key.as_ptr() as *mut _,
            ))
        }
    }

    pub fn key_switch_from(&mut self, input: &Self, key: &TlweKSKey) {
        unsafe {
            mosfhet_sys::tlwe_keyswitch(
                self.ptr,
                input.ptr,
                key.as_ptr() as *mut _,
            )
        }
    }

    pub fn key_switch(&self, key: &TlweKSKey) -> Self {
        let lower_n = key.out_lower_n();
        let mut output = unsafe { Self::new_uninit(lower_n) };
        output.key_switch_from(self, key);
        output
    }
}

impl_load!(Tlwe => tlwe_load_new_sample(lower_n: u32));
impl_save!(Tlwe => tlwe_save_sample);
impl_ptrs!(Tlwe);

unsafe impl Send for Tlwe {}
unsafe impl Sync for Tlwe {}

impl Clone for Tlwe {
    fn clone(&self) -> Self {
        let lower_n = self.lower_n();
        let mut output = unsafe { Self::new_uninit(lower_n) };
        output.clone_from(self);
        output
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe { mosfhet_sys::tlwe_copy(self.ptr, source.ptr) }
    }
}

impl Drop for Tlwe {
    fn drop(&mut self) {
        unsafe { mosfhet_sys::free_tlwe(self.ptr) }
    }
}

pub struct TlweArray {
    len: usize,
    ptr: *mut mosfhet_sys::TLWE,
}

impl TlweArray {
    pub(crate) unsafe fn new_uninit(len: usize, lower_n: u32) -> Self {
        let lower_n = lower_n as i32;
        let ptr = mosfhet_sys::tlwe_alloc_sample_array(len as i32, lower_n);
        Self { len, ptr }
    }

    pub fn from_fn<F>(len: usize, key: &TlweKey, f: F) -> Self
    where
        F: Fn(usize) -> Torus,
    {
        let mut output = unsafe { Self::new_uninit(len, key.lower_n()) };
        output
            .as_slice_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| p.set(f(i), key));
        output
    }

    pub fn from_fn_noiseless<F>(len: usize, lower_n: u32, f: F) -> Self
    where
        F: Fn(usize) -> Torus,
    {
        let mut output = unsafe { Self::new_uninit(len, lower_n) };
        output
            .as_slice_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| p.set_noiseless(f(i)));
        output
    }

    pub fn from_elem(len: usize, key: &TlweKey, elem: Torus) -> Self {
        Self::from_fn(len, key, |_| elem)
    }

    pub fn from_elem_noiseless(len: usize, lower_n: u32, elem: Torus) -> Self {
        Self::from_fn_noiseless(len, lower_n, |_| elem)
    }

    pub fn zeroed(len: usize, key: &TlweKey) -> Self {
        Self::from_elem(len, key, Torus::MIN)
    }

    pub fn zeroed_noiseless(len: usize, lower_n: u32) -> Self {
        Self::from_elem_noiseless(len, lower_n, Torus::MIN)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn lower_n(&self) -> u32 {
        unsafe { (*(*self.ptr)).n as u32 }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tlwe> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Tlwe> {
        self.as_slice_mut().iter_mut()
    }
}

impl_save_array!(TlweArray => tlwe_save_sample);
impl_drop_array!(TlweArray => free_tlwe_array);
impl_slice_array!(TlweArray);
impl_ptrs!(TlweArray);

unsafe impl Send for TlweArray {}
unsafe impl Sync for TlweArray {}

impl Clone for TlweArray {
    fn clone(&self) -> Self {
        unsafe {
            let mut output = Self::new_uninit(self.len, self.lower_n());
            output.clone_from(self);
            output
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.as_slice_mut().clone_from_slice(source.as_slice());
    }
}

impl Index<usize> for TlweArray {
    type Output = Tlwe;

    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl IndexMut<usize> for TlweArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.as_slice_mut().index_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_arithmetic() {
        let lower_n = 632;
        let sigma = 3.0517578125e-05;
        let key = TlweKey::new_binary(lower_n, sigma);

        let a = Tlwe::new(Torus::from_double(0.1), &key);
        let b = Tlwe::new(Torus::from_double(0.2), &key);
        let c = Tlwe::new_noiseless(Torus::from_double(0.3), lower_n);

        let r = Tlwe::add(&a, &b);
        let d = r
            .phase(&key)
            .distance(Torus::from_double(0.3))
            .into_double();
        assert!(dbg!(d) <= 0.001);

        let r = Tlwe::sub(&c, &a);
        let d = r
            .phase(&key)
            .distance(Torus::from_double(0.2))
            .into_double();
        assert!(dbg!(d) <= 0.001);

        let mut r = Tlwe::new(Torus::from_double(0.2), &key);
        r.add_assign(&c);
        let d = r
            .phase(&key)
            .distance(Torus::from_double(0.5))
            .into_double();
        assert!(dbg!(d) <= 0.001);

        let mut r = Tlwe::new(Torus::from_double(0.3), &key);
        r.sub_assign(&a);
        let d = r
            .phase(&key)
            .distance(Torus::from_double(0.2))
            .into_double();
        assert!(dbg!(d) <= 0.001);
    }

    #[test]
    fn unsigned_arithmetic() {
        let lower_n = 632;
        let sigma = 3.0517578125e-05;
        let key = TlweKey::new_binary(lower_n, sigma);

        let a = Tlwe::new(Torus::from_unsigned(1, 3), &key);
        let b = Tlwe::new(Torus::from_unsigned(2, 3), &key);
        let c = Tlwe::new_noiseless(Torus::from_unsigned(3, 3), lower_n);

        let r = Tlwe::add(&a, &b);
        let d = r
            .phase(&key)
            .distance(Torus::from_unsigned(3, 3))
            .into_double();
        assert!(dbg!(d) <= 0.001);

        let r = Tlwe::sub(&c, &a);
        let d = r
            .phase(&key)
            .distance(Torus::from_unsigned(2, 3))
            .into_double();
        assert!(dbg!(d) <= 0.001);

        let mut r = Tlwe::new(Torus::from_unsigned(2, 3), &key);
        r.add_assign(&c);
        let d = r
            .phase(&key)
            .distance(Torus::from_unsigned(5, 3))
            .into_double();
        assert!(dbg!(d) <= 0.001);

        let mut r = Tlwe::new(Torus::from_unsigned(2, 3), &key);
        r.sub_assign(&a);
        let d = r
            .phase(&key)
            .distance(Torus::from_unsigned(1, 3))
            .into_double();
        assert!(dbg!(d) <= 0.001);

        let r = a.neg();
        let d = r
            .phase(&key)
            .distance(Torus::from_unsigned(7, 3))
            .into_double();
        assert!(dbg!(d) <= 0.001);
    }

    #[test]
    fn new_load_save() {
        let lower_n = 632;
        let sigma = 3.0517578125e-05;
        let path = "/tmp/__tlwe_sample";
        let key = TlweKey::new_binary(lower_n, sigma);

        let a = Tlwe::new(Torus::from_double(0.2), &key);
        a.save(path).unwrap();
        let b = Tlwe::load(path, lower_n).unwrap();
        std::fs::remove_file(path).unwrap();

        assert_eq!(a.lower_n(), b.lower_n());
        let d = a.phase(&key).distance(b.phase(&key)).into_double();
        assert!(dbg!(d) <= (4.0 * sigma));
    }

    #[test]
    fn array_sum() {
        let lower_n = 632;
        let sigma = 3.0517578125e-05;
        let key = TlweKey::new_binary(lower_n, sigma);

        let len = 8usize;
        let elem = Torus::from_double(0.5 / (len as f64));
        let arr = TlweArray::from_elem(len, &key, elem);
        let mut sum = Tlwe::zero_noiseless(lower_n);
        sum.set_noiseless(Torus::MIN);

        for item in arr.as_slice().iter() {
            sum.add_assign(item);
        }

        let d = sum
            .phase(&key)
            .distance(Torus::from_double(0.5))
            .into_double();
        assert!(dbg!(d) <= (10.0 * sigma));
    }

    #[test]
    fn from_trlwe() {
        use crate::common::RawTorus;
        use crate::poly::TorusPolynomial;
        use crate::trlwe::TrlweKey;
        let upper_n = 1024;
        let k = 1;
        let sigma = 5.51342964172363e-08;
        let log_scale = 11;
        let trlwe_key = TrlweKey::new(upper_n, k, sigma);
        let tlwe_key = TlweKey::from_trlwe_key(&trlwe_key);
        let poly = TorusPolynomial::from_fn(upper_n, |i| {
            Torus::from_unsigned(i as RawTorus, log_scale)
        });
        let trlwe = Trlwe::new(poly, &trlwe_key);
        let tlwe = Tlwe::from_trlwe(&trlwe, 42);
        assert_eq!(tlwe.phase(&tlwe_key).into_unsigned(log_scale), 42);
    }

    #[test]
    fn key_switch() {
        let lower_n_1 = 630;
        let sigma_1 = 3.0517578125e-05;
        let lower_n_2 = 1024;
        let sigma_2 = 5.51342964172363e-08;
        let t = 2;
        let base_bit = 6;
        let tlwe_key_1 = TlweKey::new_binary(lower_n_1, sigma_1);
        let tlwe_key_2 = TlweKey::new_binary(lower_n_2, sigma_2);
        let tlwe_ks_key = TlweKSKey::new(&tlwe_key_1, &tlwe_key_2, t, base_bit);
        let tlwe_1 = Tlwe::new(Torus::from_unsigned(1, 3), &tlwe_key_1);
        let tlwe_2 = tlwe_1.key_switch(&tlwe_ks_key);
        assert_eq!(tlwe_2.phase(&tlwe_key_2).into_unsigned(3), 1);
    }
}
