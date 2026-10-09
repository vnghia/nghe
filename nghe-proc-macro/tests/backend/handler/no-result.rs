use nghe_proc_macro::handler;

#[handler]
pub async fn handler(database: &Database, request: Request) -> Response {
    let content = "function body should be kept";
    Response { a: true, b: 1, c: "c" }
}
