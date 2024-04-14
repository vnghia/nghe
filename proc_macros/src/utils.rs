use std::sync::OnceLock;

use concat_string::concat_string;
use syn::spanned::Spanned;
use syn::{parse_str, Error, Expr, ExprPath, Fields, Ident, Item};

pub fn get_base_name(ident: &Ident, suffix: &str) -> Result<String, Error> {
    ident
        .to_string()
        .strip_suffix(suffix)
        .ok_or_else(|| {
            Error::new(ident.span(), concat_string!("struct's name should end with ", suffix))
        })
        .map(String::from)
}

pub fn get_caller_types_module() -> ExprPath {
    parse_str::<ExprPath>(
        &std::path::Path::new("nghe_types")
            .join(
                proc_macro::Span::call_site()
                    .source_file()
                    .path()
                    .with_extension("")
                    .strip_prefix("src")
                    .unwrap()
                    .strip_prefix("open_subsonic")
                    .unwrap(),
            )
            .components()
            .map(|c| {
                if let std::path::Component::Normal(c) = c {
                    c.to_string_lossy()
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>()
            .join("::"),
    )
    .unwrap()
}

pub fn expr_to_string(e: &Expr) -> Result<String, Error> {
    if let Expr::Path(p) = e {
        Ok(p.path
            .segments
            .last()
            .ok_or(Error::new(p.span(), "last path segment is missing"))?
            .ident
            .to_string())
    } else {
        Err(Error::new(e.span(), "expression should be a path"))
    }
}

pub fn all_roles() -> &'static Vec<String> {
    static ALL_ROLES: OnceLock<Vec<String>> = OnceLock::new();
    ALL_ROLES.get_or_init(|| {
        let file = syn::parse_file(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .join("src")
                    .join("models")
                    .join("users.rs"),
            )
            .unwrap(),
        )
        .unwrap();
        let role_struct = file
            .items
            .into_iter()
            .find_map(|i| {
                if let Item::Struct(item) = i
                    && item.ident == "Role"
                {
                    Some(item)
                } else {
                    None
                }
            })
            .unwrap();
        if let Fields::Named(fields) = role_struct.fields {
            fields
                .named
                .into_iter()
                .map(|n| n.ident.unwrap().to_string().strip_suffix("_role").unwrap().to_string())
                .collect()
        } else {
            unreachable!()
        }
    })
}
