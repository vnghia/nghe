use leptos::prelude::*;

#[repr(transparent)]
#[derive(Debug, Clone, Default)]
pub struct PendingHide(pub bool);

#[repr(transparent)]
#[derive(Debug, Clone, Default)]
pub struct Error(pub Option<String>);

impl PendingHide {
    pub fn signal() -> ReadSignal<Self> {
        let (read, write) = signal(Self::default());
        provide_context(write);
        read
    }

    fn context() -> WriteSignal<Self> {
        use_context().expect("Toast error pending hide context should be provided")
    }

    pub fn set() {
        Self::context()(Self(true));
    }

    pub fn clear() {
        Self::context()(Self(false));
    }
}

impl Error {
    pub fn signal() -> ReadSignal<Self> {
        let (read, write) = signal(Self::default());
        provide_context(write);
        read
    }

    fn context() -> WriteSignal<Self> {
        use_context().expect("Toast error context should be provided")
    }

    pub fn set(inner: String) {
        PendingHide::clear();
        Self::context()(Self(Some(inner)));
    }

    pub fn clear() {
        Self::context()(Self(None));
    }
}
