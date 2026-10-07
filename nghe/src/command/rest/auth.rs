use clap::Args;

#[derive(Debug, Args)]
pub struct Auth {
    #[arg(long)]
    username: Option<String>,
    #[arg(long)]
    password: Option<String>,
    #[arg(long)]
    api_key: Option<String>,
}
