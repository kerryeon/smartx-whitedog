pub mod zeus;

#[derive(Clone, Debug, Serialize, Deserialize, Clap)]
#[clap(about = "A command which to do.")]
pub enum SubCommand {
    #[clap(subcommand)]
    Zeus(self::zeus::SubCommandZeus),
}

impl SubCommand {
    pub async fn exec(self) -> anyhow::Result<()> {
        match self {
            Self::Zeus(e) => e.exec().await,
        }
    }
}
