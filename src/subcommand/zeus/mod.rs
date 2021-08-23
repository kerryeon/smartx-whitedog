use anyhow::Result;
use reqwest::RequestBuilder;

mod apc_dir_purc_aply_e;

#[derive(Clap)]
#[clap(about = "GIST ZEUS System")]
pub enum SubCommandZeus {
    #[clap(subcommand)]
    Buy(self::apc_dir_purc_aply_e::SubCommandZeusApcDirPurcAplyE),
}

impl SubCommandZeus {
    pub async fn exec(&self) -> Result<()> {
        let client = ZeusClient::infer().await?;
        match self {
            Self::Buy(e) => e.exec(&client).await,
        }
    }
}

pub struct ZeusClient {
    client: reqwest::Client,
}

impl ZeusClient {
    /// 환경변수를 이용하여 자동 로그인합니다.
    pub async fn infer() -> Result<Self> {
        let id = std::env::var("ZEUS_ID")?;
        let password = std::env::var("ZEUS_PASSWORD")?;

        let client = reqwest::Client::builder().cookie_store(true).build()?;
        let url = format!("{}{}", Self::origin(), "/sys/login/auth.do?callback=");
        let builder = client.post(url).form(&json!({
            "login_id": &id,
            "login_pw": &password,
        }));

        // let builder = builder.header("Host", Self::host());
        // let builder = builder.header("Origin", Self::origin());
        let builder = builder.header("Referer", "https://zeus.gist.ac.kr/sys/main/login.do");
        // let builder = builder.header(
        //     "User-Agent",
        //     "Mozilla/5.0 (X11; Linux x86_64; rv:86.0) Gecko/20100101 Firefox/86.0",
        // );
        // let builder = builder.header("X-Requested-With", "XMLHttpRequest");
        // let builder = builder.header("Accept", "application/json, text/javascript, */*; q=0.01");
        // let builder = builder.header("Accept-Encoding", "gzip, deflate, br");
        // let builder = builder.header("Accept-Language", "en-US,en;q=0.5");
        // let builder = builder.header(
        //     "Content-Type",
        //     "application/x-www-form-urlencoded; charset=UTF-8",
        // );
        // let builder = builder.header("DNT", "1");

        type Map = serde_json::Map<String, serde_json::Value>;

        let response = builder.send().await?;
        let text = response.text().await?;
        let data: Option<Map> = serde_json::from_str(&text).ok();
        if data
            .as_ref()
            .filter(|e| {
                e.get("error_msg")
                    .filter(|e| {
                        if let serde_json::Value::String(e) = e {
                            e.is_empty()
                        } else {
                            false
                        }
                    })
                    .is_some()
            })
            .is_some()
        {
            Ok(Self { client })
        } else {
            bail!("failed to login to ZEUS: {}", text)
        }
    }

    pub fn attach_payload(
        builder: RequestBuilder,
        payload: serde_json::Value,
    ) -> Result<RequestBuilder> {
        const SEP: &str = "\u{001e}";

        match payload {
            serde_json::Value::Object(e) => {
                let payload = e
                    .into_iter()
                    .map(|(k, v)| format!("{}={}", k, v.to_string()))
                    .collect::<Vec<_>>()
                    .join(SEP);
                let payload = format!("SSV:utf-8{}{}", SEP, payload);
                Ok(builder.body(payload))
            }
            payload => bail!("failed to parse the payload: {:?}", payload),
        }
    }

    pub const fn host() -> &'static str {
        "zeus.gist.ac.kr"
    }

    pub const fn origin() -> &'static str {
        "https://zeus.gist.ac.kr"
    }
}
