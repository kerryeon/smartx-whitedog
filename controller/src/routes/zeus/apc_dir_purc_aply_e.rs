use chrono::{Duration, Utc};
use rocket::{serde::json::Json, State};
use ya_gist_core::models::{status::Status, zeus::apc_dir_purc_aply_e as model};

use crate::api::*;
use crate::status::ToResponse;

pub fn mount(builder: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    builder.mount(model::common::RESOURCE_URI, routes![get])
}

#[get("/?<request..>")]
async fn get(
    client: &State<super::ZeusClient>,
    request: model::get::Request,
) -> Json<Status<model::get::Response>> {
    request.exec(client).await.to_response()
}

#[async_trait]
impl GetRequest for model::get::Request {
    type Client = super::ZeusClient;

    type Response = model::get::Response;

    async fn exec(&self, client: &Self::Client) -> anyhow::Result<Self::Response> {
        let now = Utc::now();

        let mbr_no = &client.user().mbr_no;
        let aply_fr_dt = self.aply_to_dt.as_ref().cloned().unwrap_or_else(|| {
            (now - Duration::weeks(4))
                .format(Self::Client::DATETIME_FORMAT)
                .to_string()
        });
        let aply_to_dt = self
            .aply_to_dt
            .as_ref()
            .cloned()
            .unwrap_or_else(|| now.format(Self::Client::DATETIME_FORMAT).to_string());

        let response: Vec<serde_json::Value> = client
            .get(
                "/apc/apcDirPurcAplyE/selectMain.do",
                Some("PERS01^PERS01_15^002^ApcDirPurcAplyE"),
                json!({
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
                    "purc_aply_no": "",
                    "purc_prog_st_cd": "",
                }),
            )
            .await?;
        dbg!(response);
        todo!()
    }
}
