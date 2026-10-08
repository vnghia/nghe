use conf::Conf;

#[derive(Debug, Conf)]
pub struct Auth {
    #[arg(long)]
    username: Option<String>,
    #[arg(long)]
    password: Option<String>,
    #[arg(long)]
    api_key: Option<String>,
}
