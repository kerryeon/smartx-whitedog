use anyhow::Result;
use reqwest::{Method, Url};
use serde::{de::DeserializeOwned, Serialize};

use crate::models::status::Status;

pub struct Client {
    inner: reqwest::Client,
    origin: String,
}

impl Client {
    pub fn new_localhost_debug() -> Self {
        Self {
            inner: reqwest::Client::new(),
            origin: "http://127.0.0.1:8000/".to_string(),
        }
    }

    pub async fn get<Req, Res>(&self, resource_uri: &str, request: &Req) -> Result<Res>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        self.call(Method::GET, resource_uri, request).await
    }

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
