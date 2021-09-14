use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DateTime(pub chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }

    pub fn weeks_ago(weeks: u32) -> Self {
        Self(chrono::Utc::now() - chrono::Duration::weeks(weeks as i64))
    }

    pub fn format(&self, fmt: DateTimeFormat) -> String {
        self.0.format(fmt.as_str()).to_string()
    }
}

#[cfg(feature = "rocket")]
#[rocket::async_trait]
impl<'v> rocket::form::FromFormField<'v> for DateTime {
    fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
        match field.value.parse() {
            Ok(e) => Ok(Self(e)),
            Err(_) => Err(field.unexpected())?,
        }
    }

    async fn from_data(field: rocket::form::DataField<'v, '_>) -> rocket::form::Result<'v, Self> {
        Err(field.unexpected())?
    }

    fn default() -> Option<Self> {
        None
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateTimeFormat {
    YYYYMMDD,
}

impl fmt::Display for DateTimeFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl DateTimeFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::YYYYMMDD => "%Y%m%d",
        }
    }
}
