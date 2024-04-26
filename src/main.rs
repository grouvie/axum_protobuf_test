use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, Method, Response, Uri},
    middleware::{self, Next},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use axum_extra::protobuf::Protobuf;
use ctx::Ctx;
use error::MyResult;
use serde_json::json;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::{error::MyError, log::log_request, model::ModelController};
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Test {
    #[prost(int64, required, tag = "1")]
    pub id: i64,
    #[prost(string, required, tag = "2")]
    pub text: ::prost::alloc::string::String,
}

mod ctx;
mod error;
mod log;
mod model;

#[tokio::main]
async fn main() {
    let mc = ModelController::new();

    let routes_all = Router::new()
        .route("/test", post(test))
        .layer(middleware::from_fn_with_state(mc, mw_ctx_resolver))
        .layer(middleware::map_response(main_response_mapper));

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Binding TcpListener failed");
    println!("->> LISTENING on {:?}\n", listener.local_addr());
    axum::serve(listener, routes_all).await.unwrap();
}

async fn test(Protobuf(payload): Protobuf<Test>) -> MyResult<Protobuf<Test>> {
    println!("->> {:<12} - login", "HANDLER");

    println!("id: {}", payload.id);

    println!("text: {}", payload.text);

    let response = Test {
        id: 1,
        text: "Success".to_string(),
    };

    Ok(Protobuf(response))
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response<Body>,
) -> Response<Body> {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<MyError>();
    let client_status_error = service_error.map(MyError::client_status_and_error);

    // -- If client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            println!("    ->> client_error_body: {client_error_body}");

            // Build the new response from the client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error)
        .await
        .unwrap();

    println!();
    error_response.unwrap_or(res)
}

async fn mw_ctx_resolver(
    _mc: State<ModelController>,
    mut req: Request<Body>,
    next: Next,
) -> MyResult<Response<Body>> {
    // Store the ctx_result in the request extension.
    req.extensions_mut().insert(1);

    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = MyError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> MyResult<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<MyResult<Ctx>>()
            .ok_or(MyError::AuthFailCtxNotInRequestExt)?
            .clone()
    }
}
