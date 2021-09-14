#[cfg(feature = "rocket")]
use rocket::serde::json::Json;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum Status<T> {
    Success { data: T },
    Err { message: String },
}

#[cfg(feature = "rocket")]
pub trait ToResponse<T> {
    fn to_response(self) -> Json<Status<T>>;
}

#[cfg(feature = "rocket")]
impl<T> ToResponse<T> for anyhow::Result<T> {
    fn to_response(self) -> Json<Status<T>> {
        match self {
            Ok(data) => Json(Status::Success { data }),
            Err(e) => {
                error!("internal error: {}\n{:#?}", &e, &e);
                Json(Status::Err {
                    message: e.to_string(),
                })
            }
        }
    }
}
