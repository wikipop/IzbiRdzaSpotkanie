//! Contract tests: the full user flow over HTTP against the monolith, which
//! mounts the same gateway and nano routers as production.
//!
//! Each test talks to the system the way the frontend does: gateway calls
//! go to `/…`, slot calls go to `/slots/{slot}/…`.

use std::time::Duration;

use auth::{DEFAULT_ISSUER, TokenIssuer, TokenVerifier, generate_keypair};
use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

struct System {
    app: Router,
    _data: TempDir,
}

fn system() -> System {
    let data = TempDir::new().unwrap();
    let pair = generate_keypair();
    let issuer = TokenIssuer::new(
        "k1",
        &pair.private_pem,
        DEFAULT_ISSUER,
        Duration::from_secs(3600),
    )
    .unwrap();
    let verifier =
        TokenVerifier::new(&[("k1".to_owned(), pair.public_pem)], DEFAULT_ISSUER).unwrap();
    System {
        app: monolith::app(data.path(), issuer, verifier).unwrap(),
        _data: data,
    }
}

impl System {
    async fn call(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        let body = match body {
            Some(json) => {
                request = request.header("Content-Type", "application/json");
                Body::from(json.to_string())
            }
            None => Body::empty(),
        };
        let response = self
            .app
            .clone()
            .oneshot(request.body(body).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn create_event(&self, slots: &[&str]) -> String {
        let (status, body) = self
            .call(
                Method::POST,
                "/events",
                None,
                Some(json!({ "name": "Contract", "slots": slots })),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        body["event_id"].as_str().unwrap().to_owned()
    }

    async fn join(&self, eid: &str, body: Value) -> (StatusCode, Value) {
        self.call(
            Method::POST,
            &format!("/events/{eid}/join"),
            None,
            Some(body),
        )
        .await
    }

    async fn token(&self, eid: &str, nick: &str) -> String {
        let (status, body) = self.join(eid, json!({ "nick": nick })).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body["token"].as_str().unwrap().to_owned()
    }

    async fn put(&self, slot: &str, eid: &str, token: &str, state: &str) -> (StatusCode, Value) {
        self.call(
            Method::PUT,
            &format!("/slots/{slot}/events/{eid}/me"),
            Some(token),
            Some(json!({ "state": state })),
        )
        .await
    }

    async fn delete(&self, slot: &str, eid: &str, token: &str) -> StatusCode {
        self.call(
            Method::DELETE,
            &format!("/slots/{slot}/events/{eid}/me"),
            Some(token),
            None,
        )
        .await
        .0
    }

    /// Public read of one slot, without `Authorization`.
    async fn entries(&self, slot: &str, eid: &str) -> Vec<Value> {
        let (status, body) = self
            .call(
                Method::GET,
                &format!("/slots/{slot}/events/{eid}"),
                None,
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["slot"], slot);
        body["entries"].as_array().unwrap().clone()
    }
}

#[tokio::test]
async fn full_availability_lifecycle() {
    let sys = system();
    let eid = sys.create_event(&["pn-09", "pn-10", "wt-18"]).await;

    let (status, event) = sys
        .call(Method::GET, &format!("/events/{eid}"), None, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(event["slots"], json!(["pn-09", "pn-10", "wt-18"]));

    let wiktor = sys.token(&eid, "Wiktor").await;
    let ola = sys.token(&eid, "Ola").await;

    // PUT into several slots.
    for slot in ["pn-09", "pn-10"] {
        let (status, entry) = sys.put(slot, &eid, &wiktor, "yes").await;
        assert_eq!(status, StatusCode::OK, "{entry}");
        assert_eq!(entry["state"], "yes");
    }
    sys.put("pn-10", &eid, &ola, "maybe").await;

    // GET returns entries with nicks, without any token.
    let pn09 = sys.entries("pn-09", &eid).await;
    assert_eq!(pn09.len(), 1);
    assert_eq!(pn09[0]["nick"], "Wiktor");
    assert_eq!(pn09[0]["state"], "yes");
    let mut pn10: Vec<String> = sys
        .entries("pn-10", &eid)
        .await
        .iter()
        .map(|e| {
            format!(
                "{}:{}",
                e["nick"].as_str().unwrap(),
                e["state"].as_str().unwrap()
            )
        })
        .collect();
    pn10.sort();
    assert_eq!(pn10, ["Ola:maybe", "Wiktor:yes"]);
    assert!(sys.entries("wt-18", &eid).await.is_empty());

    // Edit yes -> maybe.
    let (status, entry) = sys.put("pn-09", &eid, &wiktor, "maybe").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(entry["state"], "maybe");
    let pn09 = sys.entries("pn-09", &eid).await;
    assert_eq!(pn09.len(), 1);
    assert_eq!(pn09[0]["state"], "maybe");

    // DELETE, then GET is empty; DELETE is idempotent.
    assert_eq!(
        sys.delete("pn-09", &eid, &wiktor).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        sys.delete("pn-09", &eid, &wiktor).await,
        StatusCode::NO_CONTENT
    );
    assert!(sys.entries("pn-09", &eid).await.is_empty());
}

#[tokio::test]
async fn rejoining_without_password_returns_the_same_participant() {
    let sys = system();
    let eid = sys.create_event(&["pn-09"]).await;

    let (_, first) = sys.join(&eid, json!({ "nick": "Ala" })).await;
    let (status, second) = sys.join(&eid, json!({ "nick": "ALA" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["user_id"], second["user_id"]);

    // A token from "another device" edits the same entry.
    let first_token = first["token"].as_str().unwrap();
    let second_token = second["token"].as_str().unwrap();
    sys.put("pn-09", &eid, first_token, "yes").await;
    sys.put("pn-09", &eid, second_token, "maybe").await;
    let entries = sys.entries("pn-09", &eid).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["state"], "maybe");
}

#[tokio::test]
async fn rejoining_with_password_requires_the_password() {
    let sys = system();
    let eid = sys.create_event(&["pn-09"]).await;

    let (status, first) = sys
        .join(&eid, json!({ "nick": "Ola", "password": "hunter2" }))
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, again) = sys
        .join(&eid, json!({ "nick": "Ola", "password": "hunter2" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["user_id"], again["user_id"]);

    let (status, body) = sys
        .join(&eid, json!({ "nick": "Ola", "password": "nope" }))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");

    let (status, _) = sys.join(&eid, json!({ "nick": "Ola" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn writing_to_a_slot_outside_the_event_is_forbidden() {
    let sys = system();
    let eid = sys.create_event(&["pn-09"]).await;
    let token = sys.token(&eid, "Wiktor").await;

    let (status, body) = sys.put("pn-10", &eid, &token, "yes").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "slot_out_of_range");
    assert_eq!(
        sys.delete("nd-23", &eid, &token).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn token_of_one_event_cannot_write_to_another() {
    let sys = system();
    let a = sys.create_event(&["pn-09"]).await;
    let b = sys.create_event(&["pn-09"]).await;
    let token_a = sys.token(&a, "Wiktor").await;

    let (status, body) = sys.put("pn-09", &b, &token_a, "yes").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "event_mismatch");
    assert!(sys.entries("pn-09", &b).await.is_empty());
}

#[tokio::test]
async fn writes_require_a_token() {
    let sys = system();
    let eid = sys.create_event(&["pn-09"]).await;
    let (status, body) = sys
        .call(
            Method::PUT,
            &format!("/slots/pn-09/events/{eid}/me"),
            None,
            Some(json!({ "state": "yes" })),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "missing_token");
}

#[tokio::test]
async fn every_slot_is_mounted_with_health() {
    let sys = system();
    for slot in ["pn-00", "sr-09", "nd-23"] {
        let (status, body) = sys
            .call(Method::GET, &format!("/slots/{slot}/health"), None, None)
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({ "slot": slot, "status": "ok" }));
    }
    let (status, _) = sys.call(Method::GET, "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
}
