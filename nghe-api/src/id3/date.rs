use nghe_proc_macro::api_derive;

#[api_derive(test(fake = false))]
#[derive(Default)]
pub struct Date {
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl Date {
    pub fn is_none(&self) -> bool {
        self.year.is_none()
    }
}

#[cfg(any(test, feature = "test"))]
#[coverage(off)]
mod test {
    use fake::{Dummy, Fake, Faker};

    use super::*;

    impl Dummy<Faker> for Date {
        fn dummy_with_rng<R: fake::rand::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
            let date: time::Date = config.fake_with_rng(rng);

            let year = if config.fake_with_rng(rng) {
                Some(date.year().clamp(0, 9999).try_into().unwrap())
            } else {
                None
            };
            let month = if year.is_some() && config.fake_with_rng(rng) {
                Some(date.month().into())
            } else {
                None
            };
            let day =
                if month.is_some() && config.fake_with_rng(rng) { Some(date.day()) } else { None };

            Self { year, month, day }
        }
    }
}
