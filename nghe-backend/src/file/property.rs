#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy, Clone, Copy))]
pub struct File<F> {
    pub hash: u64,
    pub size: u32,
    pub format: F,
}
