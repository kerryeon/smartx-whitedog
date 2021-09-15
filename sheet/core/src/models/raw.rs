use super::metadata::Metadata;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RawField<F> {
    #[serde(flatten)]
    pub metadata: Metadata,
    #[serde(flatten)]
    pub data: F,
}
