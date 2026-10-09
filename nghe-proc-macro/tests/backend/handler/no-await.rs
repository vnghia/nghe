use nghe_proc_macro::handler;

#[handler]
pub fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let content = "function body should be kept";
    Response { a: true, b: 1, c: "c" }
}
