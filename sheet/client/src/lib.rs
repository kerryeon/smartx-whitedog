#[macro_use]
extern crate anyhow;

use std::{
    env, fmt,
    marker::PhantomData,
    ops::{self, Index},
    str::FromStr,
};

use anyhow::Result;
use google_sheets4::{
    api::{SpreadsheetMethods, ValueRange},
    Sheets,
};
use hyper_rustls::HttpsConnector;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use yup_oauth2::ServiceAccountAuthenticator;

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
    pub async fn get_table<Field>(&self, fields_range: impl ToString) -> Result<Table<'_, Field>>
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
            Some(object) => {
                let fields_range = fields_range.to_string();
                let fields_struct: Vec<_> = object.properties.into_iter().map(|(k, _)| k).collect();
                let fields_matrix = self.get(&fields_range).await?;
                dbg!(&fields_matrix.size);
                println!("{}", &fields_matrix.size);
                dbg!(fields_matrix.data.len());

                for row in &fields_matrix.data {
                    dbg!(row);
                }

                Ok(Table {
                    spreadsheet: self,
                    name,
                    fields: todo!(),
                    fields_range,
                    _table: PhantomData::<Field>::default(),
                })
            }
            None => bail!(
                "field {} is not a struct (not supported: enum, union, ...)",
                name
            ),
        }
    }

    async fn get(&self, range: &str) -> Result<Matrix> {
        let (_, ret) = self.client.values_get(&self.id, range).doit().await?;
        Ok(Matrix {
            size: ret.range.expect("range").parse()?,
            data: ret.values.expect("values"),
        })
    }

    async fn update(&self, matrix: Matrix) -> Result<()> {
        let range = matrix.size.to_string();
        let value_range = ValueRange {
            major_dimension: None,
            range: Some(range.clone()),
            values: Some(matrix.data),
        };

        self.client
            .values_update(value_range, &self.id, &range)
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
        dbg!(range);
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct Matrix {
    size: MatrixSize,
    data: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct MatrixSize {
    sheet: String,
    start: MatrixIndex,
    end: MatrixIndex,
}

impl FromStr for MatrixSize {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split("!");
        let sheet = iter.next().unwrap();
        let grid = iter
            .next()
            .ok_or_else(|| anyhow!("sheet name is required (ex: MySheet!A1:B2)"))?;
        if let Some(_) = iter.next() {
            bail!("malformed MatrixSize: {}", s);
        }

        let mut iter = grid.split(":");
        let pos1 = iter.next().unwrap();
        let pos2 = iter.next().unwrap_or(pos1);

        let mut pos = [pos1.parse()?, pos2.parse()?];
        pos.sort();
        let [start, end] = pos;

        Ok(Self {
            sheet: sheet.to_string(),
            start,
            end,
        })
    }
}

impl fmt::Display for MatrixSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.sheet.fmt(f)?;
        "!".fmt(f)?;
        self.start.fmt(f)?;
        if &self.start != &self.end {
            ":".fmt(f)?;
            self.end.fmt(f)?;
        }
        Ok(())
    }
}

impl<'a> Index<&'a MatrixIndex> for MatrixSize {
    type Output = String;

    fn index(&self, index: &'a MatrixIndex) -> &Self::Output {
        todo!()
    }
}

impl MatrixSize {
    pub fn width(&self) -> u16 {
        self.end.col - self.start.col
    }

    pub fn height(&self) -> u32 {
        self.end.row - self.start.row
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MatrixIndex {
    col: u16,
    row: u32,
}

impl FromStr for MatrixIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            bail!("non-ascii code in MatrixIndex");
        }

        let mut col: u16 = 0;
        let mut bytes = s.bytes();
        for byte in &mut bytes {
            if byte.is_ascii_alphabetic() {
                col = col * Self::NUM_ALPHABETS + (byte.to_ascii_uppercase() - b'A') as u16;
                if col >= 17_576 {
                    bail!("columns over 'ZZZ' (17,576) are not supported");
                }
            } else {
                let row: u32 = String::from_utf8([byte].into_iter().chain(bytes).collect())
                    .unwrap()
                    .parse()?;
                if row == 0 {
                    bail!("rows with zero index are not supported");
                } else if row > 5_000_000 {
                    bail!("rows over 5 million are not supported");
                }
                return Ok(Self { col, row: row - 1 });
            }
        }
        bail!("malformed MatrixIndex: {}", s);
    }
}

impl fmt::Display for MatrixIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut col = self.col;
        while {
            (((col % Self::NUM_ALPHABETS) as u8 + b'A') as char).fmt(f)?;
            col /= Self::NUM_ALPHABETS;
            col > 0
        } {}
        (self.row + 1).fmt(f)?;
        Ok(())
    }
}

impl ops::Add for MatrixIndex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            col: self.col + rhs.col,
            row: self.row + rhs.row,
        }
    }
}

impl ops::Sub for MatrixIndex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            col: self.col - rhs.col,
            row: self.row - rhs.row,
        }
    }
}

impl MatrixIndex {
    const NUM_ALPHABETS: u16 = (b'Z' - b'A' + 1) as u16;
}
