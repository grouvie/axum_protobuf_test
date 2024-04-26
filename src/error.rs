use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use strum::AsRefStr;

pub type MyResult<T> = core::result::Result<T, MyError>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
pub enum MyError {
    ExampleError,
    AuthFailCtxNotInRequestExt,
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        // Create a placeholder Axum response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response.
        response.extensions_mut().insert(self);

        response
    }
}

impl MyError {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            // -- Example
            Self::ExampleError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::EXAMPLE_ERROR,
            ),
            // -- Auth
            Self::AuthFailCtxNotInRequestExt => (StatusCode::FORBIDDEN, ClientError::AUTH),
            // -- Fallback
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    AUTH,
    EXAMPLE_ERROR,
    SERVICE_ERROR,
}
