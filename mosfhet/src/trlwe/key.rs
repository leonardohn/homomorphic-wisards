use crate::common::macros::*;
use crate::tlwe::TlweKey;

#[repr(transparent)]
pub struct TrlweKey {
    ptr: mosfhet_sys::TRLWE_Key,
}

impl TrlweKey {
    pub fn new(upper_n: u32, k: u32, sigma: f64) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trlwe_new_binary_key(
                    upper_n as i32,
                    k as i32,
                    sigma,
                )
            },
        }
    }

    pub fn k(&self) -> u32 {
        unsafe { (*self.ptr).k as u32 }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*(*(*self.ptr).s)).N as u32 }
    }

    pub fn sigma(&self) -> f64 {
        unsafe { (*self.ptr).sigma }
    }
}

impl_load!(TrlweKey => trlwe_load_new_key);
impl_save!(TrlweKey => trlwe_save_key);
impl_drop!(TrlweKey => free_trlwe_key);
impl_ptrs!(TrlweKey);

unsafe impl Send for TrlweKey {}
unsafe impl Sync for TrlweKey {}

#[repr(transparent)]
pub struct TrlweKSKey {
    ptr: mosfhet_sys::TRLWE_KS_Key,
}

impl TrlweKSKey {
    pub fn new(
        from_key: &TrlweKey,
        into_key: &TrlweKey,
        t: u32,
        base_bit: u32,
    ) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trlwe_new_KS_key(
                    into_key.as_ptr() as *mut _,
                    from_key.as_ptr() as *mut _,
                    t as i32,
                    base_bit as i32,
                )
            },
        }
    }

    pub fn in_k(&self) -> u32 {
        unsafe { (*self.ptr).k as u32 }
    }

    pub fn out_k(&self) -> u32 {
        unsafe { (*(*(*(*self.ptr).s))).k as u32 }
    }

    pub fn out_upper_n(&self) -> u32 {
        unsafe { (*(*(*(*(*(*self.ptr).s))).a)).N as u32 }
    }
}

impl_load!(TrlweKSKey => trlwe_load_new_KS_key);
impl_save!(TrlweKSKey => trlwe_save_KS_key);
impl_drop!(TrlweKSKey => free_trlwe_ks_key);
impl_ptrs!(TrlweKSKey);

unsafe impl Send for TrlweKSKey {}
unsafe impl Sync for TrlweKSKey {}

#[repr(transparent)]
pub struct TrlwePKSKey {
    ptr: mosfhet_sys::TRLWE_KS_Key,
}

impl TrlwePKSKey {
    pub fn new(
        from_key: &TlweKey,
        into_key: &TrlweKey,
        t: u32,
        base_bit: u32,
    ) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trlwe_new_full_packing_KS_key(
                    into_key.as_ptr() as *mut _,
                    from_key.as_ptr() as *mut _,
                    t as i32,
                    base_bit as i32,
                )
            },
        }
    }

    pub fn in_k(&self) -> u32 {
        unsafe { (*self.ptr).k as u32 }
    }

    pub fn out_k(&self) -> u32 {
        unsafe { (*(*(*(*self.ptr).s))).k as u32 }
    }

    pub fn out_upper_n(&self) -> u32 {
        unsafe { (*(*(*(*(*(*self.ptr).s))).a)).N as u32 }
    }
}

impl_load!(TrlwePKSKey => trlwe_load_new_KS_key);
impl_save!(TrlwePKSKey => trlwe_save_KS_key);
impl_drop!(TrlwePKSKey => free_trlwe_ks_key);
impl_ptrs!(TrlwePKSKey);

unsafe impl Send for TrlwePKSKey {}
unsafe impl Sync for TrlwePKSKey {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_load_save() {
        let upper_n = 1024;
        let sigma = 3.0517578125e-05;
        let k = 1;
        let path = "/tmp/__trlwe_key";
        let key1 = TrlweKey::new(upper_n, k, sigma);
        key1.save(path).unwrap();
        let key2 = TrlweKey::load(path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(key1.upper_n(), key2.upper_n());
        assert_eq!(key1.sigma(), key2.sigma());
        assert_eq!(key1.k(), key2.k());
    }
}
