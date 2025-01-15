use leptos::html;
use leptos::prelude::*;

use crate::Error;

pub fn Error<I: Send + Sync + 'static>(
    action: Action<I, Result<(), Error>, SyncStorage>,
) -> impl IntoView {
    move || {
        if let Some(Err(error)) = action.value().get() {
            leptos::logging::error!("{:?}", error);
            Some(
                html::div()
                    .role("submit-alert")
                    .class(
                        "p-4 text-red-800 border border-red-300 rounded-lg bg-red-50 \
                         dark:bg-gray-800 dark:text-red-400 dark:border-red-800",
                    )
                    .child(error.to_string()),
            )
        } else {
            None
        }
    }
}
