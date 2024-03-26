use crate::common::macros::*;
use crate::trlwe::TrlweKey;

#[repr(transparent)]
pub struct TrgswKey {
    ptr: mosfhet_sys::TRGSW_Key,
}

impl TrgswKey {
    pub fn new(trlwe_key: &TrlweKey, l: u32, bg_bit: u32) -> Self {
        Self {
            ptr: unsafe {
                mosfhet_sys::trgsw_new_key(
                    trlwe_key.as_ptr() as *mut _,
                    l as i32,
                    bg_bit as i32,
                )
            },
        }
    }

    pub fn l(&self) -> u32 {
        unsafe { (*self.ptr).l as u32 }
    }

    pub fn bg_bit(&self) -> u32 {
        unsafe { (*self.ptr).Bg_bit as u32 }
    }

    pub fn k(&self) -> u32 {
        unsafe { (*(*self.ptr).trlwe_key).k as u32 }
    }

    pub fn upper_n(&self) -> u32 {
        unsafe { (*(*(*(*self.ptr).trlwe_key).s)).N as u32 }
    }

    pub fn sigma(&self) -> f64 {
        unsafe { (*(*self.ptr).trlwe_key).sigma }
    }
}

impl_load!(TrgswKey => trgsw_load_new_key);
impl_save!(TrgswKey => trgsw_save_key);
impl_drop!(TrgswKey => free_trgsw_key);
impl_ptrs!(TrgswKey);

unsafe impl Send for TrgswKey {}
unsafe impl Sync for TrgswKey {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_load_save() {
        let upper_n = 1024;
        let sigma = 3.0517578125e-05;
        let bg_bit = 24;
        let l = 1;
        let k = 1;
        let path = "/tmp/__trgsw_key";
        let key1 = TrlweKey::new(upper_n, k, sigma);
        let key2 = TrgswKey::new(&key1, l, bg_bit);
        key2.save(path).unwrap();
        let key3 = TrgswKey::load(path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(key2.l(), key3.l());
        assert_eq!(key2.bg_bit(), key3.bg_bit());
        assert_eq!(key2.upper_n(), key3.upper_n());
        assert_eq!(key2.sigma(), key3.sigma());
        assert_eq!(key2.k(), key3.k());
    }
}
