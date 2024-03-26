use std::ops::{Index, IndexMut};

use crate::common::macros::*;
use crate::common::Torus;
use crate::trgsw::TrgswKey;

#[repr(transparent)]
pub struct Trgsw {
    ptr: mosfhet_sys::TRGSW,
}

impl Trgsw {
    pub(crate) unsafe fn new_uninit(
        l: u32,
        bg_bit: u32,
        k: u32,
        upper_n: u32,
    ) -> Self {
        Self {
            ptr: mosfhet_sys::trgsw_alloc_new_sample(
                l as i32,
                bg_bit as i32,
                k as i32,
                upper_n as i32,
            ),
        }
    }

    pub fn new(m: Torus, e: u32, key: &TrgswKey) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trgsw_new_monomial_sample(
                    m.0.try_into().unwrap(),
                    e as i32,
                    key.as_ptr() as *mut _,
                )
            },
        }
    }

    pub fn set(&mut self, m: Torus, e: u32, key: &TrgswKey) {
        unsafe {
            mosfhet_sys::trgsw_monomial_sample(
                self.ptr,
                m.0.try_into().unwrap(),
                e as i32,
                key.as_ptr() as *mut _,
            )
        }
    }

    pub fn new_noiseless(
        m: Torus,
        l: u32,
        bg_bit: u32,
        k: u32,
        upper_n: u32,
    ) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trgsw_new_noiseless_trivial_sample(
                    m.0,
                    l as i32,
                    bg_bit as i32,
                    k as i32,
                    upper_n as i32,
                )
            },
        }
    }

    pub fn set_noiseless(&mut self, m: Torus) {
        unsafe {
            mosfhet_sys::trgsw_noiseless_trivial_sample(
                self.ptr,
                m.0,
                self.l() as i32,
                self.bg_bit() as i32,
                self.k() as i32,
                self.upper_n() as i32,
            )
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

    pub fn add_assign(&mut self, rhs: &Self) {
        unsafe { mosfhet_sys::trgsw_addto(self.ptr, rhs.ptr) }
    }

    pub fn add_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::trgsw_add(self.ptr, lhs.ptr, rhs.ptr) }
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
        unsafe { mosfhet_sys::trgsw_sub(self.ptr, lhs.ptr, rhs.ptr) }
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

    pub fn mul_by_xai_from(&mut self, source: &Self, a: u32) {
        unsafe { mosfhet_sys::trgsw_mul_by_xai(self.ptr, source.ptr, a as i32) }
    }

    pub fn mul_by_xai(&self, a: u32) -> Self {
        let l = self.l();
        let bg_bit = self.bg_bit();
        let k = self.k();
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(l, bg_bit, k, upper_n) };
        output.mul_by_xai_from(self, a);
        output
    }

    pub fn mul_by_xai_pred_from(&mut self, source: &Self, a: u32) {
        unsafe {
            mosfhet_sys::trgsw_mul_by_xai_minus_1(
                self.ptr, source.ptr, a as i32,
            )
        }
    }

    pub fn mul_by_xai_pred(&self, a: u32) -> Self {
        let l = self.l();
        let bg_bit = self.bg_bit();
        let k = self.k();
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(l, bg_bit, k, upper_n) };
        output.mul_by_xai_pred_from(self, a);
        output
    }

    pub fn mul_by_xai_add_assign(&mut self, source: &Self, a: u32) {
        unsafe {
            mosfhet_sys::trgsw_mul_by_xai_addto(self.ptr, source.ptr, a as i32)
        }
    }
}

impl_load!(Trgsw => trgsw_load_new_sample(l: u32, bg_bit: u32, k: u32, lower_n: u32));
impl_save!(Trgsw => trgsw_save_sample);
impl_drop!(Trgsw => free_trgsw);
impl_ptrs!(Trgsw);

unsafe impl Send for Trgsw {}
unsafe impl Sync for Trgsw {}

impl Clone for Trgsw {
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
        unsafe { mosfhet_sys::trgsw_copy(self.ptr, source.ptr) }
    }
}

pub struct TrgswArray {
    len: usize,
    ptr: *mut mosfhet_sys::TRGSW,
}

impl TrgswArray {
    pub(crate) unsafe fn new_uninit(
        len: usize,
        l: u32,
        bg_bit: u32,
        k: u32,
        upper_n: u32,
    ) -> Self {
        let ptr = mosfhet_sys::trgsw_alloc_new_sample_array(
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
                p.set(m, e, key);
            });
        output
    }

    pub fn from_fn_noiseless<F>(
        len: usize,
        l: u32,
        bg_bit: u32,
        k: u32,
        upper_n: u32,
        f: F,
    ) -> Self
    where
        F: Fn(usize) -> (Torus, u32),
    {
        let mut output =
            unsafe { Self::new_uninit(len, l, bg_bit, k, upper_n) };
        output
            .as_slice_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                let (m, e) = f(i);
                p.set_noiseless(m);
                if e != 0 {
                    // TODO: support exponents
                    unimplemented!();
                }
            });
        output
    }

    pub fn from_elem(len: usize, key: &TrgswKey, m: Torus, e: u32) -> Self {
        Self::from_fn(len, key, |_| (m, e))
    }

    pub fn from_elem_noiseless(
        len: usize,
        l: u32,
        bg_bit: u32,
        k: u32,
        upper_n: u32,
        m: Torus,
        e: u32,
    ) -> Self {
        Self::from_fn_noiseless(len, l, bg_bit, k, upper_n, |_| (m, e))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Trgsw> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Trgsw> {
        self.as_slice_mut().iter_mut()
    }
}

impl_load_array!(
    TrgswArray => trgsw_load_sample(l: u32, bg_bit: u32, k: u32, lower_n: u32)
);
impl_save_array!(TrgswArray => trgsw_save_sample);
impl_drop_array!(TrgswArray => free_trgsw_array);
impl_slice_array!(TrgswArray);
impl_ptrs!(TrgswArray);

unsafe impl Send for TrgswArray {}
unsafe impl Sync for TrgswArray {}

impl Clone for TrgswArray {
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

impl Index<usize> for TrgswArray {
    type Output = Trgsw;

    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl IndexMut<usize> for TrgswArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.as_slice_mut().index_mut(index)
    }
}
