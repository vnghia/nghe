use leptos::html;
use leptos::prelude::*;

pub fn Error<I: Send + Sync + 'static>(
    action: Action<I, Result<(), String>, SyncStorage>,
) -> impl IntoView {
    move || {
        if let Some(Err(error)) = action.value().get() {
            Some(
                html::div()
                    .role("submit-alert")
                    .class(
                        "p-4 text-red-800 border border-red-300 rounded-lg bg-red-50 \
                         dark:bg-gray-800 dark:text-red-400 dark:border-red-800",
                    )
                    .child(html::div().child(error)),
            )
        } else {
            None
        }
    }
}
