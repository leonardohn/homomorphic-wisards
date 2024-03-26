use std::ops::{Add, AddAssign, Sub, SubAssign};

pub type RawTorus = mosfhet_sys::Torus;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Torus(pub(crate) RawTorus);

impl Torus {
    pub const HALF: Self = Self(!(RawTorus::MAX >> 1));
    pub const MIN: Self = Self(RawTorus::MIN);
    pub const MAX: Self = Self(RawTorus::MAX);

    pub fn from_raw(value: RawTorus) -> Self {
        Self(value)
    }

    pub fn into_raw(self) -> RawTorus {
        self.0
    }

    pub fn from_unsigned(value: RawTorus, log_scale: usize) -> Self {
        assert!(log_scale < 32);
        assert!(value < (1 << log_scale));
        unsafe { Self(mosfhet_sys::int2torus(value, log_scale as i32)) }
    }

    pub fn into_unsigned(self, log_scale: usize) -> RawTorus {
        assert!(log_scale < 32);
        unsafe { mosfhet_sys::torus2int(self.0, log_scale as i32) }
    }

    pub fn from_double(value: f64) -> Self {
        assert!((0.0..1.0).contains(&value));
        unsafe { Self(mosfhet_sys::double2torus(value)) }
    }

    pub fn into_double(self) -> f64 {
        unsafe { mosfhet_sys::torus2double(self.0) }
    }

    pub fn distance(self, other: Self) -> Self {
        let diff = Self(self.0.abs_diff(other.0));
        if diff < Self::HALF {
            diff
        } else {
            Self::MAX - diff + Torus::from_raw(1)
        }
    }
}

unsafe impl Send for Torus {}
unsafe impl Sync for Torus {}

impl Add for Torus {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl AddAssign for Torus {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_add(rhs.0);
    }
}

impl Sub for Torus {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_sub(rhs.0))
    }
}

impl SubAssign for Torus {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_into_raw() {
        let value = 1;
        let torus = Torus::from_raw(value);
        assert_eq!(torus.into_raw(), value);
    }

    #[test]
    fn from_into_unsigned() {
        let value = 42;
        let log_scale = 6;
        let torus = Torus::from_unsigned(value, log_scale);
        assert_eq!(torus.into_unsigned(log_scale), value);
    }

    #[test]
    fn from_into_double() {
        let value = 0.49;
        let torus = Torus::from_double(value);
        assert_eq!(torus.into_double(), value);
        let value = 0.5;
        let torus = Torus::from_double(value);
        assert_eq!(torus.into_double(), value);
    }

    #[test]
    fn add_sub_assign() {
        let mut a = Torus::MIN;
        a += Torus::MAX;
        assert_eq!(a, Torus::MAX);
        a -= Torus::MAX;
        assert_eq!(a, Torus::MIN);
    }

    #[test]
    fn distance() {
        let a = Torus::from_raw(1);
        let b = Torus::from_raw(3);
        assert_eq!(a.distance(b).into_raw(), 2);
        let a = Torus::from_raw(3);
        let b = Torus::from_raw(1);
        assert_eq!(a.distance(b).into_raw(), 2);
        let a = Torus::MAX;
        let b = Torus::MIN;
        assert_eq!(a.distance(b).into_raw(), 1);
        let a = Torus::MIN;
        let b = Torus::MAX;
        assert_eq!(a.distance(b).into_raw(), 1);
    }

    #[test]
    #[should_panic]
    fn from_unsigned_invalid_scale() {
        let value = 42;
        let log_scale = 40;
        let _torus = Torus::from_unsigned(value, log_scale);
    }

    #[test]
    #[should_panic]
    fn from_unsigned_invalid_value() {
        let value = 64;
        let log_scale = 6;
        let _torus = Torus::from_unsigned(value, log_scale);
    }

    #[test]
    #[should_panic]
    fn from_double_invalid_positive() {
        let value = 1.0;
        let _torus = Torus::from_double(value);
    }

    #[test]
    #[should_panic]
    fn from_double_invalid_negative() {
        let value = -0.1;
        let _torus = Torus::from_double(value);
    }

    #[test]
    #[should_panic]
    fn from_double_invalid_nan() {
        let value = f64::NAN;
        let _torus = Torus::from_double(value);
    }
}
