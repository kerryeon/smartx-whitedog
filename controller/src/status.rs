use rocket::serde::json::Json;
use ya_gist_core::models::status::Status;

pub trait ToResponse<T> {
    fn to_response(self) -> Json<Status<T>>;
}

impl<T> ToResponse<T> for anyhow::Result<T> {
    fn to_response(self) -> Json<Status<T>> {
        match self {
            Ok(data) => Json(Status::Success { data }),
            Err(e) => {
                warn!("internal error: {}\n{:#?}", &e, &e);
                Json(Status::Err {
                    message: e.to_string(),
                })
            }
        }
    }
}
