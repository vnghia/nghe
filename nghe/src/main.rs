#![feature(coverage_attribute)]

use nghe::cli;

#[coverage(off)]
#[tokio::main]
async fn main() {
    cli::entrypoint().await;
}
