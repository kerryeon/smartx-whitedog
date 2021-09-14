#[macro_use]
extern crate anyhow;

use std::{env, marker::PhantomData};

use anyhow::Result;
use google_sheets4::{api::SpreadsheetMethods, Sheets};
use hyper_rustls::HttpsConnector;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use yup_oauth2::ServiceAccountAuthenticator;

pub use google_sheets4::api::ValueRange;

/// Google Sheets를 제어 가능한 클라이언트입니다.
pub struct SheetClient {
    hub: Sheets,
}

impl SheetClient {
    /// 클라이언트를 초기화합니다.
    ///
    /// ## Prerequisites
    /// 원하는 Google Sheets에 접근 가능한 Google service account가 필요합니다.
    /// 이는 GCP 프로젝트를 생성한 후, 프로젝트 내에 service account를 만드는 것으로 구현할 수 있습니다.
    /// 여기서, 생성한 service account가 원하는 sheets에 접근이 가능하도록 필요한 파일들을 해당 계정에 적절할 권한으로 공유해두어야 합니다.
    /// 이때, 생성한 계정의 이메일 주소를 활용하면 쉽게 공유가 가능합니다.
    ///
    /// ## Note
    /// 안전한 통신 및 프로그래밍을 위해, API 키 정보를 환경변수를 통해 프로그램의 외부에서 설정하도록 구현하였습니다.
    /// 이에, 이 라이브러리를 활용하는 프로그램을 수행하기 위해서는 다음의 환경변수가 필요합니다!
    /// * GOOGLE_OAUTH2_SERVICE_ACCOUNT: Google Drive에 접근 가능한 Google service account (json 파일 경로)
    pub async fn try_default() -> Result<Self> {
        // Get a service account info
        let path = env::var("GOOGLE_OAUTH2_SERVICE_ACCOUNT")?;
        let key = serde_json::from_reader(std::fs::File::open(path)?)?;

        // Instantiate the authenticator. It will choose a suitable authentication flow for you,
        // unless you replace  `None` with the desired Flow.
        // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
        // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
        // retrieve them from storage.
        let auth = ServiceAccountAuthenticator::builder(key)
            .build()
            .await
            .unwrap();
        let hub = Sheets::new(
            hyper::Client::builder().build(HttpsConnector::with_native_roots()),
            auth,
        );
        Ok(Self { hub })
    }

    pub fn get_sheet_unchecked(&self, id: impl ToString) -> Spreadsheet<'_> {
        Spreadsheet {
            client: self.hub.spreadsheets(),
            id: id.to_string(),
        }
    }
}

pub struct Spreadsheet<'a> {
    client: SpreadsheetMethods<'a>,
    id: String,
}

impl<'a> Spreadsheet<'a> {
    pub fn get_table<Field>(&self, fields_range: impl ToString) -> Result<Table<'_, Field>>
    where
        Field: Serialize + DeserializeOwned + JsonSchema,
    {
        let mut schema = schemars::schema_for!(Field).schema;
        let name = schema
            .metadata()
            .title
            .clone()
            .unwrap_or_else(|| "unknown field".to_string());
        match schema.object {
            Some(object) => Ok(Table {
                spreadsheet: self,
                name,
                fields: object.properties.into_iter().map(|(k, _)| k).collect(),
                fields_range: fields_range.to_string(),
                _table: PhantomData::<Field>::default(),
            }),
            None => bail!(
                "field {} is not a struct (not supported: enum, union, ...)",
                name
            ),
        }
    }

    async fn get(&self, range: &str) -> Result<ValueRange> {
        let (_, ret) = self.client.values_get(&self.id, range).doit().await?;
        Ok(ret)
    }

    async fn update(&self, range: ValueRange) -> Result<()> {
        let range_str = range.range.clone().ok_or_else(|| anyhow!("empty range"))?;
        self.client
            .values_update(range, &self.id, &range_str)
            .value_input_option("USER_ENTERED")
            .doit()
            .await?;
        Ok(())
    }
}

pub struct Table<'a, Field> {
    spreadsheet: &'a Spreadsheet<'a>,
    name: String,
    fields: Vec<String>,
    fields_range: String,
    _table: PhantomData<Field>,
}

impl<'a, Field> Table<'a, Field>
where
    Field: Serialize + DeserializeOwned,
{
    pub async fn get_rows(&self, length: Option<u32>) -> Result<Vec<Field>> {
        let range = self.spreadsheet.get(&self.fields_range).await?;
        // let sheets = spreadsheet.sheets.expect("sheets");
        // if sheets.len() != 1 {
        //     bail!(
        //         "0, 2 or more sheets are detected: {} => {}",
        //         &self.name,
        //         sheets.len()
        //     );
        // }
        // dbg!(&sheets[0]);
        dbg!(range);
        todo!()
    }
}
