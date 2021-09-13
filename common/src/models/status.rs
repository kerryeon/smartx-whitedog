#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum Status<T> {
    Success { data: T },
    Err { message: String },
}
