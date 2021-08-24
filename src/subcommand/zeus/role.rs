/// 사용자 정보
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct User {
    pub base_dept_cd: String,
    pub base_dept_nm: String,
    pub domfrnr_cd: String,
    pub lang_cd: String,
    /// 학번
    pub mbr_no: String,
    pub now_st_cd: String,
    pub now_st_nm: String,
    pub orgn_clsf_cd: String,
    pub posi_dept_cd: String,
    pub posi_dept_nm: String,
    pub stts_clsf_cd: String,
    pub stts_clsf_nm: String,
    pub stts_mid_cd: String,
    pub uni_clsf_cd: String,
    /// 이름
    pub user_nm: String,
    /// 영문명
    pub user_nm_eng: String,
}

impl super::ZeusClient {
    pub async fn get_user(&self) -> super::Result<User> {
        Ok(self
            .get("/sys/main/role.do", None, ())
            .await?
            .pop()
            .unwrap())
    }
}
