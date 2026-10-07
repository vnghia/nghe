#![feature(coverage_attribute)]

use nghe::server;

#[coverage(off)]
#[tokio::main]
async fn main() {
    server::start().await;
}
