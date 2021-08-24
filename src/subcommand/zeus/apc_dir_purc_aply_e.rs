use anyhow::Result;

use super::ZeusClient;

#[derive(Clone, Debug, Serialize, Deserialize, Clap)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Clap)]
#[clap(about = "구매 - (신)직접구매신청 - 조회")]
pub struct SubCommandZeusApcDirPurcAplyEGet {
    pg_key: Option<String>,
    page_open_time: Option<String>,
}

impl SubCommandZeusApcDirPurcAplyEGet {
    pub async fn exec(&self, client: &ZeusClient) -> Result<()> {
        client
            .get("/apc/apcDirPurcAplyE/selectMain.do", self)
            .await?;
        todo!()
    }
}
