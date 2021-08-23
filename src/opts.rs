#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
)]
#[clap(setting = clap::AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(subcommand)]
    command: crate::subcommand::SubCommand,
    #[clap(short, long, about = "Whether to show logs.")]
    verbose: bool,
}

impl Opts {
    pub async fn spawn() {
        let opts: Self = clap::Clap::parse();
        opts.init_logger();
        opts.exec().await
    }

    pub async fn exec(self) {
        if let Err(e) = self.command.exec().await {
            error!("Panicked: {}\n{:#?}", &e, &e);
        }
    }

    fn init_logger(&self) {
        let level = if self.verbose {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        };

        simple_logger::SimpleLogger::new()
            .env()
            .with_level(level)
            .init()
            .unwrap();
    }
}
