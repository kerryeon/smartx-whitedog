use chrono::{Duration, Utc};
use rocket::{serde::json::Json, State};
use ya_gist_core::models::{status::Status, zeus::apc_dir_purc_aply_e as model};

use crate::api::*;
use crate::status::ToResponse;

pub fn mount(builder: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    builder.mount(model::res::RESOURCE_URI, routes![get])
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
        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        struct RawResponse {
            purc_aply_no: String,
            purc_prog_st_cd: String,
            purc_aply_cd: String,
            purc_title: String,
            aply_dt: String,
            aply_studt_dept_cd: String,
            aply_studt_dept_nm: String,
            aplyt_studt_stts_no: String,
            aplyt_studt_stts_nm: String,
            aply_dept_cd: String,
            aply_dept_nm: String,
            aplyt_staff_no: String,
            aplyt_staff_nm: String,
            aplyt_tel_no: String,
            aplyt_email: String,
            insp_dept_cd: String,
            insp_dept_nm: String,
            inspr_staff_no: String,
            inspr_staff_nm: String,
            inspr_tel_no: String,
            vend_cd: String,
            deli_dmd_dt: String,
            pbil_sttl_mthd_cd: String,
            use_purp_cd: String,
            dir_purc_dlvmthd_cd: String,
            deli_plc: String,
            aply_dept_opin: String,
            remk: String,
            aply_amt: i64,
            key_str: String,
            deli_plc_cd: String,
            not_purc_detl: String,
            head_tel_no: String,
            head_staff_no: String,
            head_staff_nm: String,
            head_dept_cd: String,
            head_dept_nm: String,
            aply_studt_tel_no: String,
            atfile_mngt_no: String,
            anno_to_dt: String,
            anno_ssmi: String,
            enable_gbn: String,
            readonly_gbn: String,
            enable2_gbn: String,
            enable3_gbn: String,
            enable4_gbn: String,
            cnt: i64,
            hwak_cnt: i64,
            wan_yn: String,
            visible1_gbn: String,
            visible2_gbn: String,
            visible3_gbn: String,
            ref_tel_no: String,
            panel: String,
            crud_txt: String,
        }

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

        let response: Vec<RawResponse> = client
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
        Ok(vec![])
    }
}
