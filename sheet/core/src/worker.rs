use std::{env, marker::PhantomData, time::Duration};

use anyhow::Result;
use smartx_whitedog_sheet_client::{SheetClient, Spreadsheet};

use crate::models::metadata::Metadata;

pub use smartx_whitedog_common::worker::AsyncWorker;

/// 필드 값을 관리합니다.
#[async_trait]
pub trait FieldController {
    async fn on_update(
        self,
        last: Option<Self>,
        metadata: Option<Metadata>,
    ) -> Result<Option<Self>>
    where
        Self: Sized;
}

/// 주어진 명령에 따라 주기적으로 시트를 관리합니다.
#[derive(Clone)]
pub struct SheetWorker<F> {
    client: SheetClient,
    spreadsheet_management: Spreadsheet,
    spreadsheet_backup: Spreadsheet,
    interval: Duration,
    _field: PhantomData<F>,
}

#[async_trait]
impl<F> AsyncWorker for SheetWorker<F>
where
    F: FieldController + Sync,
{
    const NAMESPACE: &'static str = "whitedog-sheet-system";

    /// Worker를 초기화합니다.
    ///
    /// ## Note
    /// 이에, 프로그램을 수행하기 위해서는 다음의 환경변수가 필요합니다!
    /// * GOOGLE_OAUTH2_SERVICE_ACCOUNT: Google Drive에 접근 가능한 Google service account (json 파일 경로)
    /// * GOOGLE_SPREADSHEET_MANAGEMENT_ID: 관리하고자 하는 문서 ID
    /// * GOOGLE_SPREADSHEET_BACKUP_ID: 백업을 위한 문서 ID
    async fn try_new() -> Result<Self>
    where
        Self: Sized,
    {
        let client = SheetClient::try_default().await?;
        let spreadsheet_management = env::var("GOOGLE_SPREADSHEET_MANAGEMENT_ID")?;
        let spreadsheet_backup = env::var("GOOGLE_SPREADSHEET_BACKUP_ID")?;
        let interval = Duration::from_secs(10);

        let spreadsheet_management = client.clone().into_sheet_unchecked(spreadsheet_management);
        let spreadsheet_backup = client.clone().into_sheet_unchecked(spreadsheet_backup);

        Ok(Self {
            client,
            spreadsheet_management,
            spreadsheet_backup,
            interval,
            _field: Default::default(),
        })
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(10)
    }

    async fn tick(&self) -> Result<()> {
        info!("hello world");
        Ok(())
    }
}
