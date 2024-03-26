use crate::common::macros::*;
use crate::poly::DftPolynomial;
use crate::trgsw::TrgswDft;
use crate::trlwe::Trlwe;

#[cfg(feature = "rng_vaes")]
use crate::poly::TorusPolynomial;

#[cfg(feature = "rng_vaes")]
use crate::trlwe::TrlweKey;

#[repr(transparent)]
pub struct TrlweDft {
    ptr: mosfhet_sys::TRLWE_DFT,
}

impl TrlweDft {
    pub(crate) unsafe fn new_uninit(k: u32, upper_n: u32) -> Self {
        Self {
            ptr: mosfhet_sys::trlwe_alloc_new_DFT_sample(
                k as i32,
                upper_n as i32,
            ),
        }
    }

    #[cfg(feature = "rng_vaes")]
    pub fn new_compressed(m: TorusPolynomial, key: &TrlweKey) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trlwe_new_compressed_DFT_sample(
                    m.as_ptr() as *mut _,
                    key.as_ptr() as *mut _,
                )
            },
        }
    }

    pub fn new_noiseless(m: DftPolynomial, k: u32, upper_n: u32) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trlwe_new_noiseless_trivial_DFT_sample(
                    m.as_ptr() as *mut _,
                    k as i32,
                    upper_n as i32,
                )
            },
        }
    }

    pub fn set_noiseless(&mut self, m: DftPolynomial) {
        unsafe {
            mosfhet_sys::trlwe_noiseless_trivial_DFT_sample(
                self.ptr,
                m.as_ptr() as *mut _,
            )
        }
    }

    #[cfg(feature = "rng_vaes")]
    pub fn zeroed(key: &TrlweKey) -> Self {
        let upper_n = key.upper_n();
        let m = TorusPolynomial::zeroed(upper_n);
        Self::new_compressed(m, key)
    }

    pub fn zeroed_noiseless(k: u32, upper_n: u32) -> Self {
        let m = DftPolynomial::zeroed(upper_n);
        Self::new_noiseless(m, k, upper_n)
    }

    pub fn k(&self) -> u32 {
        unsafe { (*self.ptr).k as u32 }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*(*(*self.ptr).a)).N as u32 }
    }

    pub fn add_assign(&mut self, rhs: &Self) {
        unsafe { mosfhet_sys::trlwe_DFT_addto(self.ptr, rhs.ptr) }
    }

    pub fn add_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::trlwe_DFT_add(self.ptr, lhs.ptr, rhs.ptr) }
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

    pub fn sub_from(&mut self, lhs: &Self, rhs: &Self) {
        unsafe { mosfhet_sys::trlwe_DFT_sub(self.ptr, lhs.ptr, rhs.ptr) }
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

    pub fn mul_poly_from(&mut self, lhs: &Self, rhs: &DftPolynomial) {
        unsafe {
            mosfhet_sys::trlwe_DFT_mul_by_polynomial(
                self.ptr,
                lhs.ptr,
                rhs.as_ptr() as *mut _,
            )
        }
    }

    pub fn mul_poly(&self, poly: &DftPolynomial) -> Self {
        let k = self.k();
        let upper_n = self.upper_n();
        assert_eq!(upper_n, poly.upper_n());
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.mul_poly_from(self, poly);
        output
    }

    pub fn mul_add_assign_poly(&mut self, other: &Self, poly: &DftPolynomial) {
        unsafe {
            mosfhet_sys::trlwe_DFT_mul_addto_by_polynomial(
                self.ptr,
                other.ptr,
                poly.as_ptr() as *mut _,
            )
        }
    }

    pub fn mul_trlwe_dft(lhs: &Trlwe, rhs: &TrgswDft) -> Self {
        let k = lhs.k();
        let upper_n = lhs.upper_n();
        assert_eq!(k, rhs.k());
        assert_eq!(upper_n, rhs.upper_n());
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.mul_trlwe_dft_from(lhs, rhs);
        output
    }

    pub fn mul_trlwe_dft_from(&mut self, lhs: &Trlwe, rhs: &TrgswDft) {
        unsafe {
            mosfhet_sys::trgsw_mul_trlwe_DFT(
                self.ptr,
                lhs.as_ptr() as *mut _,
                rhs.as_ptr() as *mut _,
            )
        }
    }
}

impl_load!(TrlweDft => trlwe_load_new_DFT_sample(k: u32, lower_n: u32));
impl_save!(TrlweDft => trlwe_save_DFT_sample);
impl_drop!(TrlweDft => free_trlwe);
impl_ptrs!(TrlweDft);

unsafe impl Send for TrlweDft {}
unsafe impl Sync for TrlweDft {}

impl Clone for TrlweDft {
    fn clone(&self) -> Self {
        let k = self.k();
        let upper_n = self.upper_n();
        let mut output = unsafe { Self::new_uninit(k, upper_n) };
        output.clone_from(self);
        output
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe { mosfhet_sys::trlwe_DFT_copy(self.ptr, source.ptr) }
    }
}
