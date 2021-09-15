#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Header {
    pub administrator: Option<String>,
    pub application: Option<String>,
    pub format: Option<String>,
    pub updated_date: Option<String>,
    pub version: Option<String>,
    pub activated: Option<bool>,
}
