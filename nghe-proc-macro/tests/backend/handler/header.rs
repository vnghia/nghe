use nghe_proc_macro::handler;

#[handler(internal = true)]
pub async fn handler(
    database: &Database,
    #[handler(header)] range: Option<Range>,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let content = "function body should be kept";
    Ok(Response { a: true, b: 1, c: "c" })
}
