use diesel::sql_function;
use diesel::sql_types::*;

sql_function!(fn random() -> Text);

sql_function!(fn coalesce(x: Nullable<Int8>, y: Nullable<Int8>) -> Int8);
