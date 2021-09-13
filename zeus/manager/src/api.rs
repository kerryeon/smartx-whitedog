#[async_trait]
pub trait GetRequest {
    type Client;

    type Response;

    async fn exec(&self, client: &Self::Client) -> anyhow::Result<Self::Response>;
}
