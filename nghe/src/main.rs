#![feature(coverage_attribute)]

use nghe::command;

#[coverage(off)]
#[tokio::main]
async fn main() {
    command::entrypoint().await;
}
