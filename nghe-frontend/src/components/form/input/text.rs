use leptos::prelude::*;
use leptos::{attr, html};

pub fn Text(
    id: &'static str,
    label: &'static str,
    r#type: &'static str,
    name: impl Into<Option<&'static str>>,
    placeholder: impl Into<Option<&'static str>>,
    value: RwSignal<String>,
    error: ReadSignal<Option<&'static str>>,
) -> impl IntoView {
    html::div().child((
        html::label()
            .r#for(id)
            .class("block mb-2 font-medium text-gray-900 dark:text-white")
            .child(label),
        html::input()
            .id(id)
            .r#type(r#type)
            .name(name.into().unwrap_or(r#type))
            .class(
                "block p-2.5 w-full text-gray-900 bg-gray-50 rounded-lg border border-gray-300 \
                 dark:placeholder-gray-400 dark:text-white dark:bg-gray-700 dark:border-gray-600 \
                 dark:focus:ring-blue-500 dark:focus:border-blue-500 focus:ring-primary-600 \
                 focus:border-primary-600",
            )
            .placeholder(placeholder.into().unwrap_or(r#type))
            .bind(attr::Value, value),
        move || {
            error().map(|error| html::p().class("mt-2 text-red-600 dark:text-red-500").child(error))
        },
    ))
}
