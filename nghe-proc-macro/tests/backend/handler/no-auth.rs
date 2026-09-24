use nghe_proc_macro::handler;

#[handler(need_auth = false)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let content = "function body should be kept";
    Ok(Response { a: true, b: 1, c: "c" })
}
