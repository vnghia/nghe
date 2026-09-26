use std::iter::Sum;
use std::ops::Add;

use diesel::sql_types::Float;
use diesel::{AsExpression, FromSqlRow};

use crate::Error;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, AsExpression, FromSqlRow)]
#[repr(transparent)]
#[diesel(sql_type = Float)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct SignedDuration(pub time::SignedDuration);

impl From<time::SignedDuration> for SignedDuration {
    fn from(value: time::SignedDuration) -> Self {
        Self(value)
    }
}

impl From<SignedDuration> for time::SignedDuration {
    fn from(value: SignedDuration) -> Self {
        value.0
    }
}

impl From<f32> for SignedDuration {
    fn from(value: f32) -> Self {
        time::SignedDuration::seconds_f32(value).into()
    }
}

impl From<SignedDuration> for f32 {
    fn from(value: SignedDuration) -> Self {
        value.0.as_seconds_f32()
    }
}

impl TryFrom<std::time::Duration> for SignedDuration {
    type Error = Error;

    fn try_from(value: std::time::Duration) -> Result<Self, Self::Error> {
        time::SignedDuration::try_from(value).map_err(Self::Error::from).map(Self::from)
    }
}

impl Add<SignedDuration> for SignedDuration {
    type Output = Self;

    fn add(self, rhs: SignedDuration) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl Sum<SignedDuration> for SignedDuration {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::add)
    }
}

pub trait Trait {
    fn duration(&self) -> SignedDuration;
}

impl Trait for SignedDuration {
    fn duration(&self) -> SignedDuration {
        *self
    }
}

impl<D: Trait> Trait for Vec<D> {
    fn duration(&self) -> SignedDuration {
        self.iter().map(D::duration).sum()
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(&[], 0.0)]
    #[case(&[100.2, 200.3], 300.5)]
    fn test_sum(#[case] durations: &[f32], #[case] result: f32) {
        // Allow microsecond mismatch.
        assert!(
            (f32::from(
                durations.iter().copied().map(SignedDuration::from).sum::<SignedDuration>()
            ) - result)
                .abs()
                < 1e-6
        );
    }
}
