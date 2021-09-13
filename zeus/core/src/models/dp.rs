/// 직접구매조회
pub mod get {
    use ya_gist_common::models::chrono::DateTime;

    /// 직접구매조회
    #[cfg_attr(feature = "rocket2", derive(rocket::FromForm))]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct Request {
        /// 시작일자
        pub date_begin: Option<DateTime>,
        /// 종료일자
        pub date_end: Option<DateTime>,
    }

    /// 직접구매목록
    pub type Response = super::res::Products;

    #[cfg(feature = "reqwest2")]
    impl Request {
        pub async fn call(&self, client: &ya_gist_common::api::Client) -> anyhow::Result<Response> {
            client.get(super::res::RESOURCE_URI, self).await
        }
    }
}

/// 리소스 정보
pub mod res {
    use ya_gist_common::models::chrono::DateTime;

    /// 리소스 경로
    pub const RESOURCE_URI: &str = "/zeus/apa/";

    /// 직접구매상품 목록
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Products {
        /// 신청번호
        pub id: String,
        /// 신청번호
        pub date: DateTime,
        /// 제목
        pub title: String,
        /// 상품 목록
        pub products: Vec<Product>,
    }

    /// 직접구매상품
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Product {
        /// 정보
        #[serde(flatten)]
        pub template: ProductTemplate,
        /// 영수증
        #[serde(flatten)]
        pub receipt: Option<ProductReceipt>,
    }

    /// 직접구매상품 정보
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ProductTemplate {
        /// 상품명
        pub name: String,
        /// 규격
        pub format: String,
        /// 단위 및 수량
        #[serde(flatten)]
        pub amount: ProductAmount,
        /// 자산등재여부
        pub is_resource: bool,
    }

    /// 직접구매상품 영수증
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ProductReceipt {
        /// 단가
        pub unit_price: u64,
        /// 공급가
        pub supply_price: u64,
        /// 부가세
        pub vat: u64,
        /// 합계
        pub total: u64,
    }

    /// 직접구매상품단위
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(tag = "unit", content = "amount")]
    pub enum ProductAmount {
        /// 개입 (개수)
        EA(u32),
    }
}
