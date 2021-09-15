#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Metadata {
    pub confirm: Option<bool>,
    pub alert: Option<String>,
    pub hash: Option<String>,
}
