use anyhow::Result;

use super::ZeusClient;

#[derive(Clap)]
#[clap(about = "구매 - (신)직접구매신청")]
pub enum SubCommandZeusApcDirPurcAplyE {
    Get(SubCommandZeusApcDirPurcAplyEGet),
}

impl SubCommandZeusApcDirPurcAplyE {
    pub async fn exec(&self, client: &ZeusClient) -> Result<()> {
        match self {
            Self::Get(e) => e.exec(client).await,
        }
    }
}

#[derive(Clap)]
#[clap(about = "구매 - (신)직접구매신청 - 조회")]
pub struct SubCommandZeusApcDirPurcAplyEGet {}

impl SubCommandZeusApcDirPurcAplyEGet {
    pub async fn exec(&self, client: &ZeusClient) -> Result<()> {
        todo!()
    }
}
