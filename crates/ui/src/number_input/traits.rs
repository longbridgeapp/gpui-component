#[cfg(feature = "rust_decimal1")]
use rust_decimal::Decimal;
use std::fmt::Debug;
use std::ops::{Add, Sub};
use std::str::FromStr;

pub trait SteppableNumber:
	Debug + Add<Output = Self> + Sub<Output = Self> + FromStr + ToString + PartialOrd + Copy + 'static
{
	fn zero() -> Self;
	fn one() -> Self;
	fn min_value() -> Self;
	fn max_value() -> Self;
	fn is_valid_number_string(s: &str) -> bool;
	fn can_add(&self, step: Self) -> ValidationPayload<Self>;
	fn can_subtract(&self, step: Self) -> ValidationPayload<Self>;
	fn from_string(s: impl AsRef<str>) -> Option<Self>;
}

pub enum ValidationPayload<T: SteppableNumber> {
	Valid,
	Invalid { remainder: T },
}

macro_rules! impl_unsigned_number {
    ($($t:ty),*) => {
        $(
            impl SteppableNumber for $t {
                fn zero() -> Self { 0 }
                fn one() -> Self { 1 }
                fn min_value() -> Self {
                    <$t>::MIN
                }
                fn max_value() -> Self {
                    <$t>::MAX
                }
                fn is_valid_number_string(s: &str) -> bool {
                    s.parse::<$t>().is_ok()
                }
                fn can_add(&self, step: Self) -> ValidationPayload<Self> {
                    match self.checked_add(step) {
                        Some(_) => ValidationPayload::Valid,
                        None => ValidationPayload::Invalid { remainder: <$t>::MAX - *self },
                    }
                }
                fn can_subtract(&self, step: Self) -> ValidationPayload<Self> {
                    match self.checked_sub(step) {
                        Some(_) => ValidationPayload::Valid,
                        None => ValidationPayload::Invalid { remainder: *self },
                    }
                }
                fn from_string(s: impl AsRef<str>) -> Option<Self> {
                    s.as_ref().parse().ok()
                }
            }
        )*
    };
}

macro_rules! impl_signed_number {
    ($($t:ty),*) => {
        $(
            impl SteppableNumber for $t {
                fn zero() -> Self { 0 }
                fn one() -> Self { 1 }
                fn min_value() -> Self {
                    <$t>::MIN
                }
                fn max_value() -> Self {
                    <$t>::MAX
                }
                fn is_valid_number_string(s: &str) -> bool {
                    s.parse::<$t>().is_ok()
                }
                fn can_add(&self, step: Self) -> ValidationPayload<Self> {
                    match self.checked_add(step) {
                        Some(_) => ValidationPayload::Valid,
                        None => ValidationPayload::Invalid { remainder: <$t>::MAX - *self },
                    }
                }
                fn can_subtract(&self, step: Self) -> ValidationPayload<Self> {
                    match self.checked_sub(step) {
                        Some(_) => ValidationPayload::Valid,
                        None => ValidationPayload::Invalid { remainder: *self - <$t>::MIN },
                    }
                }
                fn from_string(s: impl AsRef<str>) -> Option<Self> {
                    s.as_ref().parse().ok()
                }
            }
        )*
    };
}

macro_rules! impl_float_number {
    ($($t:ty),*) => {
        $(
            impl SteppableNumber for $t {
                fn zero() -> Self {
                    0.0
                }
                fn one() -> Self {
                    1.0
                }
                fn min_value() -> Self {
                    <$t>::MIN
                }
                fn max_value() -> Self {
                    <$t>::MAX
                }
                fn is_valid_number_string(s: &str) -> bool {
                    s.parse::<$t>().is_ok()
                }
                fn can_add(&self, step: Self) -> ValidationPayload<Self> {
                    let result = *self + step;
                    if result.is_finite() {
                        ValidationPayload::Valid
                    } else {
                        ValidationPayload::Invalid { remainder: <$t>::MAX - *self }
                    }
                }
                fn can_subtract(&self, step: Self) -> ValidationPayload<Self> {
                    let result = *self - step;
                    if result.is_finite() {
                        ValidationPayload::Valid
                    } else {
                        ValidationPayload::Invalid { remainder: *self - <$t>::MIN }
                    }
                }
                fn from_string(s: impl AsRef<str>) -> Option<Self> {
                    s.as_ref().parse().ok()
                }
            }
        )*
    };
}

impl_unsigned_number!(u8, u16, u32, u64, u128, usize);
impl_signed_number!(i8, i16, i32, i64, i128, isize);
impl_float_number!(f32, f64);

#[cfg(feature = "rust_decimal1")]
impl SteppableNumber for Decimal {
	fn zero() -> Self {
		Decimal::ZERO
	}
	fn one() -> Self {
		Decimal::ONE
	}
	fn min_value() -> Self {
		Decimal::MIN
	}
	fn max_value() -> Self {
		Decimal::MAX
	}
	fn is_valid_number_string(s: &str) -> bool {
		s.parse::<Decimal>().is_ok()
	}
	fn can_add(&self, step: Self) -> ValidationPayload<Self> {
		match self.checked_add(step) {
			Some(_) => ValidationPayload::Valid,
			None => ValidationPayload::Invalid {
				remainder: Decimal::MAX - *self,
			},
		}
	}
	fn can_subtract(&self, step: Self) -> ValidationPayload<Self> {
		match self.checked_sub(step) {
			Some(_) => ValidationPayload::Valid,
			None => ValidationPayload::Invalid {
				remainder: *self - Decimal::MIN,
			},
		}
	}
	fn from_string(s: impl AsRef<str>) -> Option<Self> {
		s.as_ref().parse().ok()
	}
}
