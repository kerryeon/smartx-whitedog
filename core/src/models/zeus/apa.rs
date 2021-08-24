pub mod get {
    /// 직접구매조회
    #[cfg_attr(feature = "rocket", derive(rocket::FromForm))]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct Request {
        /// 시작일자
        pub aply_fr_dt: Option<String>,
        /// 종료일자
        pub aply_to_dt: Option<String>,
    }

    /// 직접구매목록
    pub type Response = Vec<super::res::Product>;

    #[cfg(feature = "reqwest")]
    impl Request {
        pub async fn call(&self, client: &crate::api::Client) -> anyhow::Result<Response> {
            client.get(super::res::RESOURCE_URI, self).await
        }
    }
}

pub mod res {
    /// 리소스 경로
    pub const RESOURCE_URI: &str = "/zeus/apa/";

    /// 직접구매상품
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Product {
        #[serde(flatten)]
        pub inner: ProductBuy,
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

    /// 직접구매상품단위
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(tag = "unit", content = "amount")]
    pub enum ProductAmount {
        EA(usize),
    }
}
