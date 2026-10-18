//! HTTP-level tests of a single nano service router.

use std::sync::Arc;
use std::time::Duration;

use auth::{DEFAULT_ISSUER, TokenIssuer, TokenVerifier, generate_keypair};
use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use domain::{EventId, SlotBitmap, SlotId, UserId};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

struct Harness {
    app: Router,
    issuer: TokenIssuer,
    event_id: EventId,
    _dir: TempDir,
}

fn slot(s: &str) -> SlotId {
    s.parse().unwrap()
}

fn harness(slot_id: &str) -> Harness {
    let dir = TempDir::new().unwrap();
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
    let db = nano::open_database(&dir.path().join(format!("{slot_id}.sqlite"))).unwrap();
    Harness {
        app: nano::router(slot(slot_id), db, Arc::new(verifier)),
        issuer,
        event_id: EventId::generate(),
        _dir: dir,
    }
}

impl Harness {
    fn token(&self, user_id: &UserId, nick: &str) -> String {
        let slots: SlotBitmap = [slot("pn-09"), slot("pn-10")].into_iter().collect();
        self.issuer.issue(user_id, nick, &self.event_id, &slots)
    }

    async fn send(
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
        let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json)
    }

    fn entries_uri(&self) -> String {
        format!("/events/{}", self.event_id)
    }

    fn me_uri(&self) -> String {
        format!("/events/{}/me", self.event_id)
    }
}

#[tokio::test]
async fn health_reports_slot() {
    let h = harness("sr-09");
    let (status, body) = h.send(Method::GET, "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "slot": "sr-09", "status": "ok" }));
}

#[tokio::test]
async fn put_get_update_delete_roundtrip() {
    let h = harness("pn-09");
    let user = UserId::generate();
    let token = h.token(&user, "Wiktor");

    let (status, body) = h.send(Method::GET, &h.entries_uri(), None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "slot": "pn-09", "entries": [] }));

    let (status, body) = h
        .send(
            Method::PUT,
            &h.me_uri(),
            Some(&token),
            Some(json!({ "state": "yes" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], user.as_str());
    assert_eq!(body["nick"], "Wiktor");
    assert_eq!(body["state"], "yes");

    let (status, _) = h
        .send(
            Method::PUT,
            &h.me_uri(),
            Some(&token),
            Some(json!({ "state": "maybe" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = h.send(Method::GET, &h.entries_uri(), None, None).await;
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["state"], "maybe");
    assert_eq!(entries[0]["nick"], "Wiktor");

    let (status, _) = h
        .send(Method::DELETE, &h.me_uri(), Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = h
        .send(Method::DELETE, &h.me_uri(), Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "DELETE is idempotent");

    let (_, body) = h.send(Method::GET, &h.entries_uri(), None, None).await;
    assert_eq!(body["entries"], json!([]));
}

#[tokio::test]
async fn entries_of_different_users_are_independent() {
    let h = harness("pn-10");
    let alice = UserId::generate();
    let bob = UserId::generate();
    let body = Some(json!({ "state": "yes" }));
    h.send(
        Method::PUT,
        &h.me_uri(),
        Some(&h.token(&alice, "Alice")),
        body.clone(),
    )
    .await;
    h.send(Method::PUT, &h.me_uri(), Some(&h.token(&bob, "Bob")), body)
        .await;
    h.send(
        Method::DELETE,
        &h.me_uri(),
        Some(&h.token(&alice, "Alice")),
        None,
    )
    .await;

    let (_, body) = h.send(Method::GET, &h.entries_uri(), None, None).await;
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["nick"], "Bob");
}

#[tokio::test]
async fn rejects_malformed_event_id_on_read() {
    let h = harness("pn-09");
    let (status, body) = h.send(Method::GET, "/events/not-an-id", None, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_event_id");
}

#[tokio::test]
async fn rejects_writes_without_valid_token() {
    let h = harness("pn-09");
    let body = Some(json!({ "state": "yes" }));
    let (status, json) = h.send(Method::PUT, &h.me_uri(), None, body.clone()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["error"]["code"], "missing_token");

    let (status, json) = h
        .send(Method::DELETE, &h.me_uri(), Some("nope"), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["error"]["code"], "invalid_token");
}

#[tokio::test]
async fn rejects_write_to_slot_outside_event() {
    let h = harness("nd-23");
    let token = h.token(&UserId::generate(), "Wiktor");
    let (status, body) = h
        .send(
            Method::PUT,
            &h.me_uri(),
            Some(&token),
            Some(json!({ "state": "yes" })),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "slot_out_of_range");
}

#[tokio::test]
async fn rejects_invalid_state_in_common_error_format() {
    let h = harness("pn-09");
    let token = h.token(&UserId::generate(), "Wiktor");
    let (status, body) = h
        .send(
            Method::PUT,
            &h.me_uri(),
            Some(&token),
            Some(json!({ "state": "no" })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_body");
}

#[test]
fn database_uses_wal_journal() {
    let dir = TempDir::new().unwrap();
    let conn = nano::open_database(&dir.path().join("pn-00.sqlite")).unwrap();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
}
