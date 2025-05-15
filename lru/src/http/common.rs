use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

pub type ApiResult<T> = Result<T, (StatusCode, StandardApiJsonBody<()>)>;
// pub type ApiResponseResult = ApiResult<Response>;
pub type StandardApiResult<T> = ApiResult<StandardApiJsonBody<T>>;

/// still return OK
pub fn build_error_response(code: String, message: String) -> (StatusCode, StandardApiJsonBody<()>) {
    let res = StandardApiJsonBody {
        code,
        message,
        data: (),
    };
    (StatusCode::OK, res)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StandardApiJsonBody<T: Serialize> {
    pub code: String,
    pub message: String,
    pub data: T,
}

impl<T: Serialize> IntoResponse for StandardApiJsonBody<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl<T: Serialize> From<T> for StandardApiJsonBody<T> {
    fn from(value: T) -> Self {
        StandardApiJsonBody {
            code: "00000".to_string(),
            message: "success".to_string(),
            data: value,
        }
    }
}
