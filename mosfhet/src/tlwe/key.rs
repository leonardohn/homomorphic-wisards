use crate::common::macros::*;
use crate::trlwe::TrlweKey;

#[repr(transparent)]
pub struct TlweKey {
    ptr: mosfhet_sys::TLWE_Key,
}

impl TlweKey {
    pub(crate) unsafe fn new_uninit(lower_n: u32, sigma: f64) -> Self {
        let ptr = mosfhet_sys::tlwe_alloc_key(lower_n as i32, sigma);
        Self { ptr }
    }

    pub fn new_binary(lower_n: u32, sigma: f64) -> Self {
        let ptr =
            unsafe { mosfhet_sys::tlwe_new_binary_key(lower_n as i32, sigma) };
        Self { ptr }
    }

    pub fn new_bounded(lower_n: u32, bound: u64, sigma: f64) -> Self {
        assert!(bound.is_power_of_two());
        let ptr = unsafe {
            mosfhet_sys::tlwe_new_bounded_key(lower_n as i32, bound, sigma)
        };
        Self { ptr }
    }

    pub fn lower_n(&self) -> u32 {
        unsafe { (*self.ptr).n as u32 }
    }

    pub fn sigma(&self) -> f64 {
        unsafe { (*self.ptr).sigma }
    }

    pub fn from_trlwe_key(key: &TrlweKey) -> Self {
        let sigma = key.sigma();
        let lower_n = key.upper_n() * key.k();
        unsafe {
            let mut output = Self::new_uninit(lower_n, sigma);
            mosfhet_sys::trlwe_extract_tlwe_key(
                output.as_ptr_mut() as *mut _,
                key.as_ptr() as *mut _,
            );
            output
        }
    }
}

impl_load!(TlweKey => tlwe_load_new_key);
impl_save!(TlweKey => tlwe_save_key);
impl_drop!(TlweKey => free_tlwe_key);
impl_ptrs!(TlweKey);

unsafe impl Send for TlweKey {}
unsafe impl Sync for TlweKey {}

#[repr(transparent)]
pub struct TlweKSKey {
    ptr: mosfhet_sys::TLWE_KS_Key,
}

impl TlweKSKey {
    pub fn new(
        from_key: &TlweKey,
        into_key: &TlweKey,
        t: u32,
        base_bit: u32,
    ) -> Self {
        unsafe {
            Self {
                ptr: mosfhet_sys::tlwe_new_KS_key(
                    into_key.as_ptr() as *mut _,
                    from_key.as_ptr() as *mut _,
                    t as i32,
                    base_bit as i32,
                ),
            }
        }
    }

    pub fn in_lower_n(&self) -> u32 {
        unsafe { (*self.ptr).n as u32 }
    }

    pub fn out_lower_n(&self) -> u32 {
        unsafe { (*(*(*(*(*self.ptr).s)))).n as u32 }
    }

    pub fn t(&self) -> u32 {
        unsafe { (*self.ptr).t as u32 }
    }

    pub fn base_bit(&self) -> u32 {
        unsafe { (*self.ptr).base_bit as u32 }
    }
}

impl_load!(TlweKSKey => tlwe_load_new_KS_key);
impl_save!(TlweKSKey => tlwe_save_KS_key);
impl_drop!(TlweKSKey => free_tlwe_ks_key);
impl_ptrs!(TlweKSKey);

unsafe impl Send for TlweKSKey {}
unsafe impl Sync for TlweKSKey {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tlwe_key_binary_new_load_save() {
        let lower_n = 632;
        let sigma = 3.0517578125e-05;
        let path = "/tmp/__tlwe_key_binary";
        let key1 = TlweKey::new_binary(lower_n, sigma);
        key1.save(path).unwrap();
        let key2 = TlweKey::load(path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(key1.lower_n(), key2.lower_n());
        assert_eq!(key1.sigma(), key2.sigma());
    }

    #[test]
    fn tlwe_key_bounded_new_load_save() {
        let lower_n = 632;
        let bound = 2;
        let sigma = 3.0517578125e-05;
        let path = "/tmp/__tlwe_key_bounded";
        let key1 = TlweKey::new_bounded(lower_n, bound, sigma);
        key1.save(path).unwrap();
        let key2 = TlweKey::load(path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(key1.lower_n(), key2.lower_n());
        assert_eq!(key1.sigma(), key2.sigma());
    }

    #[test]
    fn tlwe_ks_key_new_load_save() {
        let lower_n = 630;
        let sigma = 3.0517578125e-05;
        let t = 2;
        let base_bit = 6;
        let path = "/tmp/__tlwe_ks_key";
        let key1 = TlweKey::new_binary(lower_n, sigma);
        let key2 = TlweKey::new_binary(lower_n, sigma);
        let ks_key1 = TlweKSKey::new(&key1, &key2, t, base_bit);
        ks_key1.save(path).unwrap();
        let ks_key2 = TlweKSKey::load(path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(ks_key1.in_lower_n(), ks_key2.in_lower_n());
        assert_eq!(ks_key1.out_lower_n(), ks_key2.out_lower_n());
        assert_eq!(ks_key1.t(), ks_key2.t());
        assert_eq!(ks_key1.base_bit(), ks_key2.base_bit());
    }

    #[test]
    fn tlwe_key_from_trlwe_key() {
        let upper_n = 1024;
        let sigma = 3.0517578125e-05;
        let k = 1;
        let key1 = TrlweKey::new(upper_n, k, sigma);
        let key2 = TlweKey::from_trlwe_key(&key1);
        assert_eq!(key1.upper_n() * key1.k(), key2.lower_n());
        assert_eq!(key1.sigma(), key2.sigma());
    }
}
