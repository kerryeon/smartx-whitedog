#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

use std::str::Split;

use anyhow::Result;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Serialize};
use ya_gist_zeus_core::models::role::User;

/// Zeus 시스템에 접속 가능한 클라이언트입니다.
pub struct ZeusClient {
    client: reqwest::Client,
    wmonid: Option<String>,
    user: Option<User>,
}

impl ZeusClient {
    pub const DATETIME_FORMAT: &'static str = "%Y%m%d";

    /// 클라이언트를 초기화합니다.
    fn try_default() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder().cookie_store(true).build()?,
            wmonid: None,
            user: None,
        })
    }

    /// 환경변수를 이용하여 자동 로그인합니다.
    pub async fn infer() -> Result<Self> {
        let mut client = Self::try_default()?;
        client.login().await?;
        client.user.replace(client.get_user().await?);
        Ok(client)
    }

    /// 사용자 정보를 불러옵니다.
    pub fn user(&self) -> &User {
        self.user.as_ref().unwrap()
    }

    /// 시스템으로부터 데이터를 요청합니다.
    pub async fn get<D, R>(
        &self,
        resource_uri: &str,
        pg_key: Option<&str>,
        data: D,
    ) -> Result<Vec<R>>
    where
        D: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}{}", Self::origin(), resource_uri);
        let builder = self.attach_payload(self.client.get(url), pg_key, data)?;

        let response = builder.send().await?;
        Payload::new(response.text().await?).to_json()
    }

    /// 사용자 정보를 불러옵니다.
    async fn get_user(&self) -> Result<User> {
        Ok(self
            .get("/sys/main/role.do", None, ())
            .await?
            .pop()
            .unwrap())
    }

    /// 시스템에 로그인합니다.
    async fn login(&mut self) -> Result<()> {
        let url = format!("{}{}", Self::origin(), "/sys/login/auth.do?callback=");
        let builder = self.client.post(url).form(&json!({
            "login_id": &std::env::var("ZEUS_ID")?,
            "login_pw": &std::env::var("ZEUS_PASSWORD")?,
        }));
        let builder = builder.header("Referer", format!("{}/sys/main/login.do", Self::origin()));

        let response = builder.send().await?;
        let wmonid = response
            .cookies()
            .find(|e| e.name() == "WMONID")
            .map(|e| e.value().to_string());
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
            self.wmonid = wmonid;
            Ok(())
        } else {
            bail!("failed to login to ZEUS: {}", text)
        }
    }

    /// 웹 요청에 데이터를 추가합니다.
    fn attach_payload<P>(
        &self,
        builder: RequestBuilder,
        pg_key: Option<&str>,
        payload: P,
    ) -> Result<RequestBuilder>
    where
        P: Serialize,
    {
        let wmonid = self.wmonid.as_ref().map(|e| e.as_str());
        Ok(builder.body(Payload::from_json(wmonid, pg_key, payload)?.into_string()))
    }

    // const fn host() -> &'static str {
    //     "zeus.gist.ac.kr"
    // }

    const fn origin() -> &'static str {
        "https://zeus.gist.ac.kr"
    }
}

/// 시스템의 자료형
struct Payload(String);

impl Payload {
    const HEADER: &'static str = "SSV:utf-8";
    const SEP: &'static str = "\u{001e}";
    const DEP: &'static str = "\u{001f}";

    /// 시스템이 해석할 수 있는 자료형을 생성합니다.
    fn new(data: String) -> Self {
        Self(data)
    }

    /// JSON 데이터로부터 자료를 해석합니다.
    fn from_json<T>(wmonid: Option<&str>, pg_key: Option<&str>, data: T) -> Result<Self>
    where
        T: Serialize,
    {
        fn parse_value(value: serde_json::Value) -> Result<String> {
            match value {
                serde_json::Value::String(e) => Ok(e),
                value => bail!("not supported type: {:?}", value),
            }
        }

        let mut data = match serde_json::to_value(data)? {
            serde_json::Value::Null => String::new(),
            serde_json::Value::Object(e) => e
                .into_iter()
                .map(|(k, v)| Ok(format!("{}={}", k, parse_value(v)?)))
                .collect::<Result<Vec<_>>>()?
                .join(Self::SEP),
            payload => bail!("failed to parse the payload: {:?}", payload),
        };

        if let Some(wmonid) = wmonid {
            data = format!("WMONID={}{}{}", wmonid, Self::SEP, data);
        }
        {
            let pg_key = pg_key.unwrap_or("");
            data = format!("pg_key={}{}{}", pg_key, Self::SEP, data);
        }
        {
            data = format!("page_open_time={}{}", Self::SEP, data);
        }

        let data = format!("{}{}{}", Self::HEADER, Self::SEP, data);
        debug!("{:?}", &data);
        Ok(Self(data))
    }

    /// JSON 데이터로 변환합니다.
    fn to_json<T>(&self) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        #[derive(Debug)]
        struct Field<'a> {
            name: &'a str,
            ty: FieldType,
        }

        impl<'a> Field<'a> {
            fn from_str(ty: &'a str) -> Result<Self> {
                let (name, ty) = try_split(ty, ":")?;
                Ok(Self {
                    name,
                    ty: FieldType::from_str(ty)?,
                })
            }

            fn try_parse_value(&self, value: &str) -> Result<(String, serde_json::Value)> {
                Ok((self.name.to_string(), self.ty.try_parse_value(value)?))
            }
        }

        #[allow(dead_code)]
        #[derive(Debug)]
        enum FieldType {
            BigDecimal { size: usize },
            String { size: usize },
        }

        impl FieldType {
            fn from_str(ty: &str) -> Result<Self> {
                let (ty, arg) = try_split(ty, "(")?;
                let size = (&arg[..arg.len() - 1]).parse()?;
                match ty {
                    "bigdecimal" => Ok(Self::BigDecimal { size }),
                    "string" => Ok(Self::String { size }),
                    _ => bail!("unknown type: {:?}", ty),
                }
            }

            fn try_parse_value(&self, value: &str) -> Result<serde_json::Value> {
                match self {
                    Self::BigDecimal { .. } => Ok(value.parse::<i64>()?.into()),
                    Self::String { .. } => Ok(value.to_string().into()),
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

        fn split_one<'a, 'b>(text: &'a str, pat: &'b str) -> (&'a str, Split<'a, &'b str>) {
            let mut iter = text.split(pat);
            let item_1 = iter.next().unwrap();
            (item_1, iter)
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

        let pat_fields = format!("{}_RowType_{}", Self::SEP, Self::DEP);
        let pat_values = format!("{}N{}", Self::SEP, Self::DEP);
        let (_headers, text) = try_split(text, &pat_fields)?;
        let (fields, values) = split_one(text, &pat_values);

        let fields: Vec<_> = fields
            .split(Self::DEP)
            .map(|e| Field::from_str(e))
            .collect::<Result<_>>()?;
        let values: Vec<_> = values
            .map(|e| {
                e.split(Self::DEP)
                    .zip(&fields)
                    .map(|(v, f)| f.try_parse_value(v))
                    .collect::<Result<JsonObject>>()
            })
            .map(|e| e.map(serde_json::Value::Object))
            .collect::<Result<_>>()?;
        debug!("{:?}", &fields);
        serde_json::from_value(serde_json::Value::Array(values))
            .map_err(|e| anyhow!("failed to parse: {}", e))
    }

    /// 문자열을 반환합니다.
    fn into_string(self) -> String {
        self.0
    }
}

type JsonObject = serde_json::Map<String, serde_json::Value>;
