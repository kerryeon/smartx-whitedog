impl super::ZeusClient {
    pub(super) async fn get_user(&self) -> super::Result<ya_gist_core::models::zeus::role::User> {
        Ok(self
            .get("/sys/main/role.do", None, ())
            .await?
            .pop()
            .unwrap())
    }
}
