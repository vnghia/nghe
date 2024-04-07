use diesel::sql_function;

sql_function!(fn random() -> Text);
