use diesel::sql_function;
use diesel::sql_types::*;

sql_function!(fn random() -> Text);

sql_function!(fn coalesce(x: Nullable<Int8>, y: Int8) -> Int8);

sql_function! {
    #[sql_name = "coalesce"]
    fn coalescef(x: Nullable<Float4>, y: Float4) -> Float4;
}

sql_function! {
    #[sql_name = "coalesce"]
    fn coalesceid(x: Nullable<Uuid>, y: Nullable<Uuid>) -> Nullable<Uuid>;
}

sql_function!(fn greatest(x: Nullable<Timestamptz>, y: Timestamptz) -> Timestamptz);

sql_function! {
    #[aggregate]
    #[sql_name = "any_value"]
    fn any_value_text(x: Text) -> Text;
}
