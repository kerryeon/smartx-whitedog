use anyhow::Result;
use chrono::{Duration, Utc};

use super::ZeusClient;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRequest {
    /// 시작일자
    pub aply_fr_dt: Option<String>,
    /// 종료일자
    pub aply_to_dt: Option<String>,
}

impl GetRequest {
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

/// 직접구매신청상품
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductBuy {
    /// 상품명
    pub name: String,
    /// 단위
    #[serde(flatten)]
    pub amount: ProductAmount,
    /// 자산등재여부
    pub is_resource: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "unit", content = "amount")]
/// 직접구매상품단위
pub enum ProductAmount {
    EA(usize),
}
