use diesel::define_sql_function;

define_sql_function!(fn random() -> Bool);
