pub mod get {
    /// 직접구매조회
    #[cfg_attr(feature = "rocket", derive(rocket::FromForm))]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Request {
        /// 시작일자
        pub aply_fr_dt: Option<String>,
        /// 종료일자
        pub aply_to_dt: Option<String>,
    }

    /// 직접구매목록
    pub type Response = Vec<ResponseItem>;

    /// 직접구매 아이템
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ResponseItem {}
}

pub mod common {
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

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(tag = "unit", content = "amount")]
    /// 직접구매상품단위
    pub enum ProductAmount {
        EA(usize),
    }
}
