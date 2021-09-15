#![feature(cell_leak)]

#[macro_use]
extern crate anyhow;

use std::{collections::BTreeMap, env, fmt, marker::PhantomData, str::FromStr};

use anyhow::Result;
use google_sheets4::{api::ValueRange, Sheets};
use hyper_rustls::HttpsConnector;
use inflector::Inflector;
use schemars::{
    schema::{InstanceType, ObjectValidation, Schema, SingleOrVec},
    JsonSchema,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
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

    pub fn into_sheet_unchecked(self, id: impl ToString) -> Spreadsheet {
        Spreadsheet {
            client: self.hub,
            id: id.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Spreadsheet {
    client: Sheets,
    id: String,
}

impl Spreadsheet {
    pub async fn get_table<Field>(&self, fields_range: impl ToString) -> Result<Table<'_, Field>>
    where
        Field: Serialize + DeserializeOwned + JsonSchema,
    {
        fn parse_type(ty: InstanceType) -> Result<InstanceType> {
            match ty {
                InstanceType::Object => bail!("child struct is not supported"),
                _ => Ok(ty),
            }
        }

        fn parse_schema(name: String, schema: Schema) -> Result<(String, InstanceType)> {
            match schema {
                Schema::Bool(_) => {
                    bail!("a trivial boolean JSON Schema is not supported: {}", name)
                }
                Schema::Object(schema) => {
                    let ty = schema
                        .instance_type
                        .ok_or_else(|| anyhow!("cannot infer the type: {}", name))
                        .and_then(|types| match types {
                            SingleOrVec::Single(ty) => Ok(*ty),
                            SingleOrVec::Vec(types) => {
                                let mut types =
                                    types.into_iter().filter(|ty| *ty != InstanceType::Null);
                                let ty = types.next().unwrap_or(InstanceType::Null);
                                match types.next() {
                                    Some(_) => {
                                        bail!(
                                            "2 or more types in one type is not supported: {}",
                                            name
                                        )
                                    }
                                    None => Ok(ty),
                                }
                            }
                        })
                        .and_then(parse_type)?;
                    Ok((name, ty))
                }
            }
        }

        fn parse_object_properties(
            object: Box<ObjectValidation>,
        ) -> Result<Vec<(String, InstanceType)>> {
            object
                .properties
                .into_iter()
                .map(|(name, schema)| parse_schema(name, schema))
                .collect()
        }

        fn parse_matrix(
            struct_name: &str,
            fields_struct: Vec<(String, InstanceType)>,
            mut matrix: Matrix,
        ) -> Result<Vec<FieldName>> {
            let fields: Vec<_> = matrix
                .cols()
                .filter_map(|names| {
                    names
                        .into_iter()
                        .map(|name| name.to_snake_case())
                        .filter_map(|name| {
                            fields_struct
                                .iter()
                                .enumerate()
                                .find(|(_, (field, _))| &name == field)
                                .map(|(index, (field, ty))| FieldName {
                                    index: index as u16,
                                    name,
                                    field: field.clone(),
                                    ty: *ty,
                                })
                        })
                        .next()
                })
                .collect();

            for (field_name, _) in fields_struct {
                if let None = fields.iter().find(|field| &field.field == &field_name) {
                    bail!(
                        "cannot find the field \"{}\" for \"{}\" on \"{}\"",
                        field_name,
                        struct_name,
                        &matrix.shape,
                    );
                }
            }
            Ok(fields)
        }

        let mut schema = schemars::schema_for!(Field).schema;
        let name = schema
            .metadata()
            .title
            .clone()
            .unwrap_or_else(|| "unknown field".to_string());
        match schema.object {
            Some(object) => {
                let fields_range = fields_range.to_string();
                let fields_struct = parse_object_properties(object)?;
                let fields_matrix = self.get(&fields_range).await?;

                Ok(Table {
                    spreadsheet: self,
                    fields_shape: fields_matrix.shape.clone(),
                    fields: parse_matrix(&name, fields_struct, fields_matrix)?,
                    name,
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
        let (_, ret) = self
            .client
            .spreadsheets()
            .values_get(&self.id, range)
            .doit()
            .await?;
        Ok(Matrix {
            shape: ret.range.expect("range").parse()?,
            data: ret.values.expect("values"),
        })
    }

    async fn update(&self, matrix: Matrix) -> Result<()> {
        let range = matrix.shape.to_string();
        let value_range = ValueRange {
            major_dimension: None,
            range: Some(range.clone()),
            values: Some(matrix.data),
        };

        self.client
            .spreadsheets()
            .values_update(value_range, &self.id, &range)
            .value_input_option("USER_ENTERED")
            .doit()
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Table<'a, Field> {
    spreadsheet: &'a Spreadsheet,
    name: String,
    fields: Vec<FieldName>,
    fields_shape: MatrixShape,
    _table: PhantomData<Field>,
}

impl<'a, Field> Table<'a, Field>
where
    Field: Serialize + DeserializeOwned,
{
    pub async fn get_rows(&self, length: Option<u32>) -> Result<Vec<Field>> {
        fn parse_col(field: &FieldName, token: String) -> Result<Value> {
            match field.ty {
                InstanceType::Null => Ok(Value::Null),
                InstanceType::Boolean => match token.to_uppercase().as_str() {
                    "TRUE" | "YES" | "Y" | "O" | "V" => Ok(Value::Bool(true)),
                    "FALSE" | "NO" | "N" | "X" => Ok(Value::Bool(false)),
                    _ => bail!(
                        "cannot parse the value into boolean \"{}\" ({})",
                        token,
                        &field.name
                    ),
                },
                InstanceType::Integer | InstanceType::Number => Ok(Value::Number(token.parse()?)),
                // TODO: to be implemented
                InstanceType::String => Ok(Value::String(token)),
                InstanceType::Array => todo!(),
                InstanceType::Object => unreachable!("Object type should be pruned"),
            }
        }

        let matrix = self
            .spreadsheet
            .get(&self.values_shape(length).to_string())
            .await?;

        // TODO: Vector notation
        matrix
            .into_iter()
            .map(|cols| {
                let fields: BTreeMap<_, _> = self
                    .fields
                    .iter()
                    .zip(cols)
                    .map(|(k, v)| Ok((&k.field, parse_col(&k, v)?)))
                    .collect::<Result<_>>()?;
                Ok(serde_json::from_value(serde_json::to_value(fields)?)?)
            })
            .collect()
    }

    fn values_start(&self) -> MatrixIndex {
        MatrixIndex {
            col: self.fields_shape.start.col,
            row: Some(self.fields_shape.end.row.unwrap() + 1),
        }
    }

    fn values_end(&self, row: Option<u32>) -> MatrixIndex {
        MatrixIndex {
            col: self.fields_shape.end.col,
            row: row.map(|row| (self.fields_shape.end.row.unwrap() + row)),
        }
    }

    fn values_shape(&self, row: Option<u32>) -> MatrixShape {
        MatrixShape {
            sheet: self.fields_shape.sheet.clone(),
            start: self.values_start(),
            end: self.values_end(row),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldName {
    index: u16,
    name: String,
    field: String,
    ty: InstanceType,
}

#[derive(Clone, Debug)]
pub struct Matrix {
    shape: MatrixShape,
    data: Vec<Vec<String>>,
}

impl Matrix {
    pub fn get(&mut self, index: MatrixIndex) -> Option<&mut String> {
        self.get_row(index.row.expect("index row should be given"))
            .and_then(|e| e.get_mut(index.col as usize))
    }

    pub fn get_row(&mut self, row: u32) -> Option<&mut [String]> {
        self.fill_default_on_row(row);
        self.data.get_mut(row as usize).map(|e| e.as_mut_slice())
    }

    pub fn get_col(&mut self, col: u16) -> Option<Vec<&mut String>> {
        if col >= self.shape.cols() {
            return None;
        }

        let rows = self.shape.rows().expect("end row should be given");
        for row in 0..rows {
            self.fill_default_on_row(row);
        }

        let mut cols = vec![];
        for row in 0..rows {
            let index = MatrixIndex {
                col,
                row: Some(row),
            };
            cols.push(unsafe { std::mem::transmute(self.get(index).unwrap()) });
        }
        Some(cols)
    }

    fn fill_default_on_row(&mut self, row: u32) {
        let row_index = row as usize;
        let row_length = self.shape.rows().unwrap_or(MatrixIndex::MAX_ROW) as usize;
        let col_length = self.shape.cols() as usize;

        if row_index < row_length {
            while self.data.len() <= row_index {
                self.data.push(Default::default());
            }
            let row = self.data.get_mut(row_index).unwrap();
            while row.len() < col_length {
                row.push(Default::default());
            }
        }
    }

    pub fn rows(&mut self) -> self::iter::RowIterMut {
        self::iter::RowIterMut::new(self)
    }

    pub fn cols(&mut self) -> self::iter::ColIterMut {
        self::iter::ColIterMut::new(self)
    }

    pub fn shape(&self) -> &MatrixShape {
        &self.shape
    }
}

impl<'a> IntoIterator for &'a mut Matrix {
    type Item = &'a mut [String];

    type IntoIter = self::iter::RowIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl IntoIterator for Matrix {
    type Item = Vec<String>;

    type IntoIter = std::vec::IntoIter<Vec<String>>;

    fn into_iter(mut self) -> Self::IntoIter {
        let rows = self.shape.rows().unwrap_or(MatrixIndex::MAX_ROW);
        for row in 0..rows {
            self.fill_default_on_row(row);
        }
        self.data.into_iter()
    }
}
pub mod iter {
    use std::cell::UnsafeCell;

    use super::Matrix;

    #[derive(Debug)]
    pub struct RowIterMut<'a> {
        _thread_lock: UnsafeCell<()>,
        matrix: &'a mut Matrix,
        index: u32,
    }

    impl<'a> RowIterMut<'a> {
        pub(super) fn new(matrix: &'a mut Matrix) -> Self {
            Self {
                _thread_lock: Default::default(),
                matrix,
                index: 0,
            }
        }
    }

    impl<'a> Iterator for RowIterMut<'a> {
        type Item = &'a mut [String];

        fn next(&mut self) -> Option<Self::Item> {
            let index = self.index;
            self.index += 1;
            unsafe { std::mem::transmute(self.matrix.get_row(index)) }
        }
    }

    #[derive(Debug)]
    pub struct ColIterMut<'a> {
        _thread_lock: UnsafeCell<()>,
        matrix: &'a mut Matrix,
        index: u16,
    }

    impl<'a> ColIterMut<'a> {
        pub(super) fn new(matrix: &'a mut Matrix) -> Self {
            Self {
                _thread_lock: Default::default(),
                matrix,
                index: 0,
            }
        }
    }

    impl<'a> Iterator for ColIterMut<'a> {
        type Item = Vec<&'a mut String>;

        fn next(&mut self) -> Option<Self::Item> {
            let index = self.index;
            self.index += 1;
            unsafe { std::mem::transmute(self.matrix.get_col(index)) }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MatrixShape {
    pub sheet: String,
    pub start: MatrixIndex,
    pub end: MatrixIndex,
}

impl FromStr for MatrixShape {
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

impl fmt::Display for MatrixShape {
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

impl MatrixShape {
    pub fn cols(&self) -> u16 {
        self.end.col - self.start.col + 1
    }

    pub fn rows(&self) -> Option<u32> {
        match (self.start.row, self.end.row) {
            (Some(start), Some(end)) => Some(end - start + 1),
            (None, Some(end)) => Some(end),
            (_, None) => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MatrixIndex {
    pub col: u16,
    pub row: Option<u32>,
}

impl FromStr for MatrixIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            bail!("empty code in MatrixIndex is not supported");
        }
        if !s.is_ascii() {
            bail!("non-ascii code in MatrixIndex");
        }

        let mut col: u16 = 0;
        let mut bytes = s.bytes();
        for byte in &mut bytes {
            if byte.is_ascii_alphabetic() {
                col = col * Self::NUM_ALPHABETS + (byte.to_ascii_uppercase() - b'A') as u16;
                if col >= Self::MAX_COL {
                    bail!("columns over 'ZZZ' (17,576) are not supported");
                }
            } else {
                let row: u32 = String::from_utf8([byte].into_iter().chain(bytes).collect())
                    .unwrap()
                    .parse()?;
                if row == 0 {
                    bail!("rows with zero index are not supported");
                } else if row > Self::MAX_ROW {
                    bail!("rows over 5 million are not supported");
                }
                return Ok(Self {
                    col,
                    row: Some(row - 1),
                });
            }
        }
        Ok(Self { col, row: None })
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
        if let Some(row) = self.row {
            (row + 1).fmt(f)?;
        }
        Ok(())
    }
}

impl MatrixIndex {
    const MAX_ROW: u32 = 5_000_000;
    const MAX_COL: u16 = 17_576;
    const NUM_ALPHABETS: u16 = (b'Z' - b'A' + 1) as u16;

    pub fn new(col: u16, row: Option<u32>) -> Self {
        Self { col, row }
    }
}
