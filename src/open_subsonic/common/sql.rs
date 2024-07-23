use diesel::define_sql_function;
use diesel::sql_types::*;

define_sql_function!(fn random() -> Text);

define_sql_function! {
    #[sql_name = "coalesce"]
    fn coalesce_i64(x: Nullable<Int8>, y: Int8) -> Int8;
}

define_sql_function! {
    #[sql_name = "coalesce"]
    fn coalesce_f32(x: Nullable<Float4>, y: Float4) -> Float4;
}

define_sql_function! {
    #[sql_name = "coalesce"]
    fn coalesce_uuid(x: Nullable<Uuid>, y: Nullable<Uuid>) -> Nullable<Uuid>;
}

define_sql_function! {
    #[sql_name = "greatest"]
    fn greatest_tz(x: Nullable<Timestamptz>, y: Timestamptz) -> Timestamptz;
}

define_sql_function! {
    #[aggregate]
    #[sql_name = "any_value"]
    fn any_value_text(x: Text) -> Text;
}
