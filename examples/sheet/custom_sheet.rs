use std::env;

use ya_gist_sheet_client::{SheetClient, ValueRange};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 환경변수를 활용해 초기화합니다.
    // * GOOGLE_SPREADSHEET_ID: 테스트하고자 하는 문서 ID
    let spreadsheet_id = env::var("GOOGLE_SPREADSHEET_ID")?;

    let client = SheetClient::try_default().await?;
    let spreadsheet = client.get_sheet_unchecked(&spreadsheet_id);
    spreadsheet
        .update_raw(ValueRange {
            major_dimension: None,
            range: Some("Sheet1!G1:G2".to_string()),
            values: Some(vec![vec!["Hello".to_string()], vec!["World".to_string()]]),
        })
        .await?;
    Ok(())
}
