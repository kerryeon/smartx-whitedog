use std::env;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ya_gist_sheet_client::{SheetClient, Table};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct MyField {
    administrator: String,
    application: String,
    format: String,
    updated_date: String,
    version: String,
    activated: bool,
    alert: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 환경변수를 활용해 초기화합니다.
    // * GOOGLE_SPREADSHEET_ID: 테스트하고자 하는 문서 ID
    let spreadsheet_id = env::var("GOOGLE_SPREADSHEET_ID")?;

    // 클라이언트를 초기화합니다.
    let client = SheetClient::try_default().await?;

    // 정의한 테이블 객체를 불러옵니다.
    let spreadsheet = client.into_sheet_unchecked(spreadsheet_id);
    let table: Table<MyField> = spreadsheet.get_table("Metadata!A1:G1").await?;
    let mut row = table
        .get_rows(Some(1))
        .await?
        .pop()
        .expect("rows should not be empty");
    dbg!(&row);

    // 객체를 수정하고 이를 반영합니다.
    row.alert = Some("hello world".to_string());
    table
        .set_rows(&[row.clone(), row.clone(), row.clone()], 0)
        .await?;

    Ok(())
}
