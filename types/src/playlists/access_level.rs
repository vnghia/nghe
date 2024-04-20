use nghe_proc_macros::add_types_derive;

#[add_types_derive]
#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessLevel {
    Read,
    Write,
    Admin,
}
