use anyhow::Result;
use reqwest::{Method, Url};
use serde::{de::DeserializeOwned, Serialize};

use crate::models::status::Status;

/// Rest API 클라이언트입니다.
pub struct Client {
    inner: reqwest::Client,
    origin: String,
}

impl Client {
    /// 로컬에서의 디버깅을 위해 클라이언트를 생성합니다.
    pub fn new_localhost_debug() -> Self {
        Self {
            inner: reqwest::Client::new(),
            origin: "http://127.0.0.1:8000/".to_string(),
        }
    }

    /// GET 요청을 수행합니다.
    pub async fn get<Req, Res>(&self, resource_uri: &str, request: &Req) -> Result<Res>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        self.call(Method::GET, resource_uri, request).await
    }

    /// 서버로부터 요청을 수행합니다.
    async fn call<Req, Res>(&self, method: Method, resource_uri: &str, request: &Req) -> Result<Res>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        let response: Status<Res> = self
            .inner
            .request(method, self.make_url(resource_uri)?)
            .json(request)
            .send()
            .await?
            .json()
            .await?;

        match response {
            Status::Success { data } => Ok(data),
            Status::Err { message } => anyhow::bail!("failed to call: {}", &message),
        }
    }

    fn make_url(&self, resource_uri: &str) -> Result<Url> {
        Ok(Url::parse(&format!("{}{}", &self.origin, resource_uri))?)
    }
}
