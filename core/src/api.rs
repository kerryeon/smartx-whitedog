use anyhow::Result;
use reqwest::{Method, Url};
use rocket::serde::DeserializeOwned;
use serde::Serialize;

pub struct Client {
    inner: reqwest::Client,
    origin: String,
}

impl Client {
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
        Ok(self
            .inner
            .request(method, self.make_url(resource_uri)?)
            .json(request)
            .send()
            .await?
            .json()
            .await?)
    }

    fn make_url(&self, resource_uri: &str) -> Result<Url> {
        Ok(Url::parse(&format!("{}{}", &self.origin, resource_uri))?)
    }
}
