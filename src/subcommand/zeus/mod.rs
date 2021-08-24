use anyhow::Result;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod role;

mod apc_dir_purc_aply_e;

#[derive(Clone, Debug, Serialize, Deserialize, Clap)]
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
    fn try_default() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder().cookie_store(true).build()?,
        })
    }

    /// 환경변수를 이용하여 자동 로그인합니다.
    pub async fn infer() -> Result<Self> {
        let client = Self::try_default()?;
        client.login().await?;
        client.get_user().await?;
        Ok(client)
    }

    pub(crate) async fn get<D, R>(&self, resource_uri: &str, data: D) -> Result<R>
    where
        D: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}{}", Self::origin(), resource_uri);
        let builder = Self::attach_payload(self.client.get(url), data)?;

        let response = builder.send().await?;
        Payload::new(response.text().await?).to_json()
    }

    async fn login(&self) -> Result<()> {
        let url = format!("{}{}", Self::origin(), "/sys/login/auth.do?callback=");
        let builder = self.client.post(url).form(&json!({
            "login_id": &std::env::var("ZEUS_ID")?,
            "login_pw": &std::env::var("ZEUS_PASSWORD")?,
        }));
        let builder = builder.header("Referer", "https://zeus.gist.ac.kr/sys/main/login.do");

        let response = builder.send().await?;
        let text = response.text().await?;
        let data: Option<JsonObject> = serde_json::from_str(&text).ok();
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
            Ok(())
        } else {
            bail!("failed to login to ZEUS: {}", text)
        }
    }

    fn attach_payload<P>(builder: RequestBuilder, payload: P) -> Result<RequestBuilder>
    where
        P: Serialize,
    {
        Ok(builder.body(Payload::from_json(payload)?.into_string()))
    }

    // const fn host() -> &'static str {
    //     "zeus.gist.ac.kr"
    // }

    const fn origin() -> &'static str {
        "https://zeus.gist.ac.kr"
    }
}

struct Payload(String);

impl Payload {
    const HEADER: &'static str = "SSV:utf-8";
    const SEP: &'static str = "\u{001e}";
    const DEP: &'static str = "\u{001f}";

    fn new(data: String) -> Self {
        Self(data)
    }

    fn from_json<T>(data: T) -> Result<Self>
    where
        T: Serialize,
    {
        match serde_json::to_value(data)? {
            serde_json::Value::Null => Ok(Self(format!("{}{}", Self::HEADER, Self::SEP))),
            serde_json::Value::Object(e) => {
                let data = e
                    .into_iter()
                    .map(|(k, v)| format!("{}={}", k, v.to_string()))
                    .collect::<Vec<_>>()
                    .join(Self::SEP);
                Ok(Self(format!("{}{}{}", Self::HEADER, Self::SEP, data)))
            }
            payload => bail!("failed to parse the payload: {:?}", payload),
        }
    }

    fn to_json<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        #[derive(Debug)]
        struct Field<'a> {
            name: &'a str,
            value: FieldValue<'a>,
        }

        impl<'a> Field<'a> {
            fn from_str(ty: &'a str, value: &'a str) -> Result<Self> {
                let (name, ty) = try_split(ty, ":")?;
                Ok(Self {
                    name,
                    value: FieldValue::from_str(ty, value)?,
                })
            }

            fn to_value(&self) -> (String, serde_json::Value) {
                (self.name.to_string(), self.value.to_value())
            }
        }

        #[derive(Debug)]
        enum FieldValue<'a> {
            String { size: usize, value: &'a str },
        }

        impl<'a> FieldValue<'a> {
            fn from_str(ty: &'a str, value: &'a str) -> Result<Self> {
                let (ty, arg) = try_split(ty, "(")?;
                let arg_len = arg.len();
                match ty {
                    "string" => Ok(Self::String {
                        size: (&arg[..arg_len - 1]).parse()?,
                        value,
                    }),
                    _ => bail!("unknown type: {:?}", ty),
                }
            }

            fn to_value(&self) -> serde_json::Value {
                match self {
                    Self::String { value, .. } => value.to_string().into(),
                }
            }
        }

        fn try_split<'a>(text: &'a str, pat: &str) -> Result<(&'a str, &'a str)> {
            let mut iter = text.split(pat);
            let item_1 = iter.next();
            let item_2 = iter.next();
            item_1.zip(item_2).ok_or_else(|| {
                anyhow!(
                    "failed to parse the payload with pattern {:?}: {:?}",
                    pat,
                    text
                )
            })
        }

        if !self.0.starts_with(Self::HEADER) {
            bail!("failed to parse the payload: {:?}", &self.0);
        }
        let text = &self.0[Self::HEADER.len()..];

        // catch an error
        if let Some(text) = text.split("ErrorMsg:string=").skip(1).next() {
            if let Some(text) = text.split(Self::SEP).next() {
                bail!("an error has occurred: {:?}", text);
            }
        }

        let (_headers, text) = try_split(text, &format!("{}_RowType_{}", Self::SEP, Self::DEP))?;
        let (fields, values) = try_split(text, &format!("{}N{}", Self::SEP, Self::DEP))?;

        let fields = fields.split(Self::DEP);
        let values = values.split(Self::DEP);
        let fields: Vec<Field> = fields
            .zip(values)
            .map(|(field, value)| Field::from_str(field, value))
            .collect::<Result<_>>()?;
        let fields = fields
            .into_iter()
            .map(|e| e.to_value())
            .collect::<JsonObject>();
        debug!("{:?}", &fields);
        serde_json::from_value(serde_json::Value::Object(fields))
            .map_err(|e| anyhow!("failed to parse: {}", e))
    }

    fn into_string(self) -> String {
        self.0
    }
}

type JsonObject = serde_json::Map<String, serde_json::Value>;
