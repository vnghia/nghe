use nghe_proc_macro::handler;

#[handler]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    #[handler(header)] range: Option<Range>,
    user_id: Uuid,
    request: Request,
) -> Result<binary::Response, Error> {
    let content = "function body should be kept";
    Ok(Response { a: true, b: 1, c: "c" })
}
