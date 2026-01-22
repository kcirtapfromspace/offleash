//! Prometheus metrics endpoint

use axum::{
    extract::State,
    http::header::CONTENT_TYPE,
    response::{IntoResponse, Response},
};

use crate::AppState;

/// GET /metrics - Prometheus metrics endpoint
pub async fn metrics(State(state): State<AppState>) -> Response {
    let metrics = state.metrics_handle.render();
    ([(CONTENT_TYPE, "text/plain; version=0.0.4")], metrics).into_response()
}
