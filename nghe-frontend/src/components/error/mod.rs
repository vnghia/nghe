mod boundary;
mod context;
mod toast;

pub use boundary::Boundary;
use leptos::prelude::*;
use leptos::{ev, html};
pub use toast::Toast;

pub fn Error() -> impl IntoView {
    let pending_hide = context::PendingHide::signal();
    let error = context::Error::signal();

    let owner = Owner::current().expect("Owner should be provided");
    move || {
        let owner = owner.clone();
        error().0.map(|error| {
            html::div()
                .role("alert")
                .class(move || {
                    if pending_hide().0 {
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
                    if !pending_hide().0 {
                        let owner = owner.clone();
                        owner.with(|| {
                            context::PendingHide::set();
                        });
                        set_timeout(
                            move || {
                                owner.with(|| {
                                    context::Error::clear();
                                });
                            },
                            std::time::Duration::from_millis(300),
                        );
                    }
                })
        })
    }
}
