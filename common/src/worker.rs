use std::time::Duration;

use anyhow::Result;
use tokio::time::{sleep, Instant};

#[async_trait]
pub trait AsyncWorker {
    const NAMESPACE: &'static str;

    async fn try_new() -> Result<Self>
    where
        Self: Sized;

    fn interval(&self) -> Duration;

    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn spawn_consuming_errors()
    where
        Self: Sized + Sync + 'static,
    {
        crate::init::init_logger();
        'main: loop {
            match Self::spawn().await {
                Ok(()) => break 'main,
                Err(error) => {
                    error!("fatal error: {err}\n- detail: {err:#?}", err = &error);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn spawn() -> Result<()>
    where
        Self: Sized + Sync + 'static,
    {
        match Self::try_new().await {
            Ok(e) => e.spawn_inner().await,
            Err(error) => panic!("fatal error: {err}\n- detail: {err:#?}", err = &error),
        }
    }

    async fn spawn_inner(self) -> Result<()>
    where
        Self: Sized + Sync + 'static,
    {
        self.init().await?;
        info!("worker inited");
        tokio::spawn(self.loop_forever()).await?
    }

    async fn loop_forever(self) -> Result<()>
    where
        Self: Sized + Sync,
    {
        loop {
            self.schedule().await?;
        }
    }

    async fn schedule(&self) -> Result<()> {
        let time_begin = Instant::now();
        let result = self.tick().await;
        let time_end = Instant::now();

        let interval = self.interval();
        let elapsed = time_end - time_begin;
        if interval > elapsed {
            sleep(interval - elapsed).await;
        }
        result
    }

    async fn schedule_without_waiting(&self) -> Result<()> {
        let result = self.tick().await;
        tokio::time::sleep(self.interval()).await;
        result
    }

    async fn tick(&self) -> Result<()> {
        Ok(())
    }
}
