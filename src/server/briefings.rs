//! `/api/days` and `/api/briefings`: what the web app lists, and marking a
//! briefing viewed, finished or unread again.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
};
use chrono::NaiveDate;
use serde::Deserialize;

use super::AppState;
use crate::{
    app::store::{DayCount, Saved, Status},
    prelude::*,
};

/// The routes, under `/api`.
pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/days", get(days))
        .route("/api/days/{day}", get(on_day))
        .route("/api/briefings/{id}/status", put(set_status))
}

/// An error answered as `{"error": "..."}`.
struct ApiError(StatusCode, String);

/// What a route answers.
type ApiResult<T> = std::result::Result<Json<T>, ApiError>;

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        tracing::error!("{error}");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "could not read or write the database".to_owned(),
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}

/// `GET /api/days`: every day with briefings, newest first, with counts.
async fn days(State(state): State<AppState>) -> ApiResult<Vec<DayCount>> {
    Ok(Json(state.store().days()?))
}

/// `GET /api/days/{day}`: that day's briefings.
async fn on_day(State(state): State<AppState>, Path(day): Path<String>) -> ApiResult<Vec<Saved>> {
    let day = NaiveDate::parse_from_str(&day, "%Y-%m-%d").map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            format!("`{day}` is not a date like 2026-11-23"),
        )
    })?;
    Ok(Json(state.store().on_day(day)?))
}

/// Body of `PUT /api/briefings/{id}/status`.
#[derive(Deserialize)]
struct StatusBody {
    /// The new read state.
    status: Status,
}

/// `PUT /api/briefings/{id}/status`: sets the read state, answers the
/// briefing. Idempotent, so a phone on flaky wifi can retry it.
async fn set_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<StatusBody>,
) -> ApiResult<Saved> {
    state
        .store()
        .set_status(id, body.status)?
        .map(Json)
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, format!("no briefing with id `{id}`")))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::{app::store::Store, server::routes};

    /// The app over a store holding one unread briefing on 2026-11-23.
    fn app() -> Router {
        let day = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap();
        routes(AppState::new(Store::sample(day)))
    }

    /// Sends `request`, answers the status and the JSON body.
    async fn send(app: Router, request: Request<Body>) -> (StatusCode, Value) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    /// A `PUT` of `body` as JSON.
    fn put_json(uri: &str, body: &Value) -> Request<Body> {
        Request::put(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn days_lists_counts() {
        let (status, body) = send(
            app(),
            Request::get("/api/days").body(Body::empty()).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            body,
            json!([{"day": "2026-11-23", "unread": 1, "viewed": 0, "finished": 0}])
        );
    }

    #[tokio::test]
    async fn on_day_lists_briefings_or_rejects_a_bad_date() {
        let request = Request::get("/api/days/2026-11-23")
            .body(Body::empty())
            .unwrap();
        let (status, body) = send(app(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body[0]["topic"], "rust");
        assert_eq!(body[0]["status"], "unread");

        let request = Request::get("/api/days/monday")
            .body(Body::empty())
            .unwrap();
        let (status, body) = send(app(), request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body["error"].as_str().unwrap().contains("monday"));
    }

    #[tokio::test]
    async fn set_status_updates_or_answers_404() {
        let app = app();
        let request = put_json("/api/briefings/1/status", &json!({"status": "finished"}));
        let (status, body) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "finished");

        let request = Request::get("/api/days").body(Body::empty()).unwrap();
        let (_, body) = send(app.clone(), request).await;
        assert_eq!(body[0]["finished"], 1);

        let request = put_json("/api/briefings/9/status", &json!({"status": "viewed"}));
        let (status, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let request = put_json("/api/briefings/1/status", &json!({"status": "read"}));
        let (status, _) = send(app, request).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}
