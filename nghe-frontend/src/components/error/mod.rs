use leptos::prelude::*;
use leptos::{ev, html};

#[repr(transparent)]
#[derive(Debug, Clone, Default)]
struct PendingHide {
    inner: bool,
}

#[repr(transparent)]
#[derive(Debug, Clone, Default)]
struct Error {
    inner: Option<String>,
}

impl PendingHide {
    fn context() -> WriteSignal<Self> {
        use_context().expect("Toast error pending hide context should be provided")
    }

    fn set() {
        Self::context()(Self { inner: true });
    }

    fn clear() {
        Self::context()(Self { inner: false });
    }
}

impl Error {
    fn context() -> WriteSignal<Self> {
        use_context().expect("Toast error context should be provided")
    }

    fn set(inner: String) {
        PendingHide::clear();
        Self::context()(Self { inner: Some(inner) });
    }

    fn clear() {
        Self::context()(Self { inner: None });
    }
}

pub fn Error() -> impl IntoView {
    let (pending_hide, set_pending_hide) = signal(PendingHide::default());
    let (error, set_error) = signal(Error::default());
    provide_context(set_pending_hide);
    provide_context(set_error);

    let owner = Owner::current().unwrap();
    move || {
        let owner = owner.clone();
        error().inner.map(|error| {
            html::div()
                .role("alert")
                .class(move || {
                    if pending_hide().inner {
                        "fixed p-4 text-red-800 rounded-lg bg-red-50 dark:bg-gray-800 \
                         dark:text-red-400 hover:ring-2 hover:ring-red-400 right-5 bottom-5 \
                         max-w-full ml-5 md:ml-69 lg:ml-0 lg:max-w-2/4 transition-opacity \
                         duration-300 ease-out opacity-0"
                    } else {
                        "fixed p-4 text-red-800 rounded-lg bg-red-50 dark:bg-gray-800 \
                         dark:text-red-400 hover:ring-2 hover:ring-red-400 right-5 bottom-5 \
                         max-w-full ml-5 md:ml-69 lg:ml-0 lg:max-w-2/4 transition-opacity \
                         duration-300 ease-out"
                    }
                })
                .child(
                    html::div()
                        .class("flex items-center justify-center")
                        .child(html::div().class("text-sm font-medium text-justify").child(error)),
                )
                .on(ev::click, move |_| {
                    if !pending_hide().inner {
                        let owner = owner.clone();
                        owner.with(|| {
                            PendingHide::set();
                        });
                        set_timeout(
                            move || {
                                owner.with(|| {
                                    Error::clear();
                                });
                            },
                            std::time::Duration::from_millis(300),
                        );
                    }
                })
        })
    }
}
