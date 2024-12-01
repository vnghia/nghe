use std::iter::Sum;
use std::ops::Add;

use diesel::sql_types::Float;
use diesel::{AsExpression, FromSqlRow};

use crate::Error;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, AsExpression, FromSqlRow)]
#[repr(transparent)]
#[diesel(sql_type = Float)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct Duration(pub time::Duration);

impl From<time::Duration> for Duration {
    fn from(value: time::Duration) -> Self {
        Self(value)
    }
}

impl From<Duration> for time::Duration {
    fn from(value: Duration) -> Self {
        value.0
    }
}

impl From<f32> for Duration {
    fn from(value: f32) -> Self {
        time::Duration::seconds_f32(value).into()
    }
}

impl From<Duration> for f32 {
    fn from(value: Duration) -> Self {
        value.0.as_seconds_f32()
    }
}

impl TryFrom<std::time::Duration> for Duration {
    type Error = Error;

    fn try_from(value: std::time::Duration) -> Result<Self, Self::Error> {
        time::Duration::try_from(value).map_err(Self::Error::from).map(Self::from)
    }
}

impl Add<Duration> for Duration {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl Sum<Duration> for Duration {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::add)
    }
}

pub trait Trait {
    fn duration(&self) -> Duration;
}

impl Trait for Duration {
    fn duration(&self) -> Duration {
        *self
    }
}

impl<D: Trait> Trait for Vec<D> {
    fn duration(&self) -> Duration {
        self.iter().map(D::duration).sum()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(&[], 0.0)]
    #[case(&[100.2, 200.3], 300.5)]
    fn test_sum(#[case] durations: &[f32], #[case] result: f32) {
        // Allow microsecond mismatch.
        assert!(
            (f32::from(durations.iter().copied().map(Duration::from).sum::<Duration>()) - result)
                .abs()
                < 1e-6
        );
    }
}
