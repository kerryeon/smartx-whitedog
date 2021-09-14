use std::env;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ya_gist_sheet_client::{SheetClient, Table};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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
    let spreadsheet = client.get_sheet_unchecked(spreadsheet_id);
    let table: Table<MyField> = spreadsheet.get_table("Metadata!A9:G12").await?;
    dbg!(table.get_rows(Some(1)).await?);

    Ok(())
}
