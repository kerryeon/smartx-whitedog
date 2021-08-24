use anyhow::Result;
use chrono::{Duration, Utc};

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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[clap(about = "구매 - (신)직접구매신청 - 조회")]
pub struct SubCommandZeusApcDirPurcAplyEGet {
    /// 시작일자
    pub aply_fr_dt: Option<String>,
    /// 종료일자
    pub aply_to_dt: Option<String>,
}

impl SubCommandZeusApcDirPurcAplyEGet {
    const DATETIME_FORMAT: &'static str = "%Y%m%d";

    pub async fn exec(&self, client: &ZeusClient) -> Result<()> {
        let now = Utc::now();

        let mbr_no = &client.user().mbr_no;
        let aply_fr_dt = self.aply_to_dt.as_ref().cloned().unwrap_or_else(|| {
            (now - Duration::weeks(4))
                .format(Self::DATETIME_FORMAT)
                .to_string()
        });
        let aply_to_dt = self
            .aply_to_dt
            .as_ref()
            .cloned()
            .unwrap_or_else(|| now.format(Self::DATETIME_FORMAT).to_string());

        let response: Vec<serde_json::Value> = client
            .get(
                "/apc/apcDirPurcAplyE/selectMain.do",
                Some("PERS01^PERS01_15^002^ApcDirPurcAplyE"),
                json!({
                    "purc_prog_st_cd": "",
                    "purc_aply_no": "",
                    "aply_dept_cd": "",
                    "aply_nm": "",
                    "aply_fr_dt": aply_fr_dt,
                    "aply_to_dt": aply_to_dt,
                    "ctrl_val1_cd": "REPM",
                    "ctrl_val2_cd": "STUD",
                    "ctrl_val3_cd": format!(
                        "AND (APA.APLYT_STUDT_STTS_NO='{mbr_no}' OR APA.HEAD_STAFF_NO='{mbr_no}' OR D.GUID_PROF_NO IN (  SELECT GUID_PROF_NO FROM USR_MST WHERE STUDT_NO='{mbr_no}') OR APA.APLYT_STUDT_STTS_NO=(  SELECT GUID_PROF_NO FROM USR_MST WHERE STUDT_NO='{mbr_no}'))",
                        mbr_no=mbr_no,
                    ),
                }),
            )
            .await?;
        dbg!(response);
        todo!()
    }
}
