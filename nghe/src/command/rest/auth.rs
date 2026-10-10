use conf::Conf;

#[derive(Debug, Conf)]
pub struct Auth {
    #[conf(long)]
    pub username: Option<String>,
    #[conf(long)]
    pub password: Option<String>,
    #[conf(long)]
    pub api_key: Option<String>,
}
