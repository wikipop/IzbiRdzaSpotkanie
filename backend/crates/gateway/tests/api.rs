//! HTTP-level tests of the gateway router.

use std::time::Duration;

use auth::{DEFAULT_ISSUER, TokenIssuer, TokenVerifier, generate_keypair};
use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use domain::{SlotBitmap, SlotId};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

struct Harness {
    app: Router,
    verifier: TokenVerifier,
    _dir: TempDir,
}

fn harness() -> Harness {
    let dir = TempDir::new().unwrap();
    let pair = generate_keypair();
    let ttl = Duration::from_secs(3600);
    let issuer = TokenIssuer::new("k1", &pair.private_pem, DEFAULT_ISSUER, ttl).unwrap();
    let verifier =
        TokenVerifier::new(&[("k1".to_owned(), pair.public_pem)], DEFAULT_ISSUER).unwrap();
    let db = gateway::open_database(&dir.path().join("gateway.sqlite")).unwrap();
    Harness {
        app: gateway::router(db, issuer),
        verifier,
        _dir: dir,
    }
}

impl Harness {
    async fn send(&self, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(uri);
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

    async fn create_event(&self, body: Value) -> String {
        let (status, json) = self.send(Method::POST, "/events", Some(body)).await;
        assert_eq!(status, StatusCode::CREATED, "{json}");
        json["event_id"].as_str().unwrap().to_owned()
    }

    async fn join(&self, eid: &str, body: Value) -> (StatusCode, Value) {
        self.send(Method::POST, &format!("/events/{eid}/join"), Some(body))
            .await
    }
}

#[tokio::test]
async fn creates_event_from_slot_list() {
    let h = harness();
    let eid = h
        .create_event(json!({ "name": "  Standup  ", "slots": ["wt-10", "pn-09", "pn-09"] }))
        .await;
    assert!(eid.starts_with("e_"));

    let (status, body) = h.send(Method::GET, &format!("/events/{eid}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({ "event_id": eid, "name": "Standup", "slots": ["pn-09", "wt-10"] })
    );
}

#[tokio::test]
async fn creates_event_from_days_and_half_open_hours() {
    let h = harness();
    let eid = h
        .create_event(
            json!({ "name": "Club", "days": ["sr", "pn"], "hours": { "from": 22, "to": 24 } }),
        )
        .await;
    let (_, body) = h.send(Method::GET, &format!("/events/{eid}"), None).await;
    assert_eq!(body["slots"], json!(["pn-22", "pn-23", "sr-22", "sr-23"]));
}

#[tokio::test]
async fn rejects_invalid_events() {
    let h = harness();
    let cases = [
        (
            json!({ "name": "", "slots": ["pn-09"] }),
            "invalid_event_name",
        ),
        (
            json!({ "name": "x".repeat(101), "slots": ["pn-09"] }),
            "invalid_event_name",
        ),
        (json!({ "name": "A", "slots": [] }), "invalid_slot"),
        (json!({ "name": "A", "slots": ["pn-24"] }), "invalid_slot"),
        (json!({ "name": "A" }), "invalid_slot"),
        (json!({ "name": "A", "days": ["pn"] }), "invalid_slot"),
        (
            json!({ "name": "A", "slots": ["pn-09"], "days": ["pn"], "hours": { "from": 9, "to": 10 } }),
            "invalid_slot",
        ),
        (
            json!({ "name": "A", "days": ["xx"], "hours": { "from": 9, "to": 10 } }),
            "invalid_slot",
        ),
        (
            json!({ "name": "A", "days": ["pn"], "hours": { "from": 10, "to": 10 } }),
            "invalid_slot",
        ),
        (
            json!({ "name": "A", "days": ["pn"], "hours": { "from": 0, "to": 25 } }),
            "invalid_slot",
        ),
        (json!({ "slots": ["pn-09"] }), "invalid_body"),
    ];
    for (body, code) in cases {
        let (status, json) = h.send(Method::POST, "/events", Some(body.clone())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(json["error"]["code"], code, "{body}");
    }
}

#[tokio::test]
async fn unknown_or_malformed_event_is_404() {
    let h = harness();
    for eid in ["e_AAAAAAAAAAAAAAAAAAAAAA", "garbage"] {
        let (status, body) = h.send(Method::GET, &format!("/events/{eid}"), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "event_not_found");
        let (status, _) = h.join(eid, json!({ "nick": "Ala" })).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn join_issues_token_with_event_scope() {
    let h = harness();
    let eid = h
        .create_event(json!({ "name": "A", "slots": ["pn-09", "sb-12"] }))
        .await;
    let (status, body) = h.join(&eid, json!({ "nick": "  Wiktor " })).await;
    assert_eq!(status, StatusCode::OK);
    let user_id = body["user_id"].as_str().unwrap();
    assert!(user_id.starts_with("u_"));

    let claims = h.verifier.verify(body["token"].as_str().unwrap()).unwrap();
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.nick, "Wiktor");
    assert_eq!(claims.eid, eid);
    let expected: SlotBitmap = ["pn-09", "sb-12"]
        .into_iter()
        .map(|s| s.parse::<SlotId>().unwrap())
        .collect();
    assert_eq!(SlotBitmap::from_base64url(&claims.slots).unwrap(), expected);
}

#[tokio::test]
async fn existing_nick_without_password_logs_in_case_insensitively() {
    let h = harness();
    let eid = h
        .create_event(json!({ "name": "A", "slots": ["pn-09"] }))
        .await;
    let (_, first) = h.join(&eid, json!({ "nick": "Łukasz" })).await;
    let (status, second) = h
        .join(&eid, json!({ "nick": "łUKASZ", "password": "ignored" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["user_id"], second["user_id"]);

    let claims = h
        .verifier
        .verify(second["token"].as_str().unwrap())
        .unwrap();
    assert_eq!(claims.nick, "Łukasz", "the original spelling is kept");
}

#[tokio::test]
async fn password_protected_nick_requires_correct_password() {
    let h = harness();
    let eid = h
        .create_event(json!({ "name": "A", "slots": ["pn-09"] }))
        .await;
    let (status, first) = h
        .join(&eid, json!({ "nick": "Ola", "password": "s3cret" }))
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, again) = h
        .join(&eid, json!({ "nick": "ola", "password": "s3cret" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["user_id"], again["user_id"]);

    for body in [
        json!({ "nick": "Ola", "password": "wrong" }),
        json!({ "nick": "Ola" }),
        json!({ "nick": "Ola", "password": "" }),
    ] {
        let (status, json) = h.join(&eid, body.clone()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
        assert_eq!(json["error"]["code"], "invalid_credentials");
    }
}

#[tokio::test]
async fn same_nick_in_different_events_is_a_different_participant() {
    let h = harness();
    let a = h
        .create_event(json!({ "name": "A", "slots": ["pn-09"] }))
        .await;
    let b = h
        .create_event(json!({ "name": "B", "slots": ["pn-09"] }))
        .await;
    let (_, in_a) = h.join(&a, json!({ "nick": "Ala" })).await;
    let (_, in_b) = h.join(&b, json!({ "nick": "Ala" })).await;
    assert_ne!(in_a["user_id"], in_b["user_id"]);
}

#[tokio::test]
async fn rejects_invalid_nick_and_password() {
    let h = harness();
    let eid = h
        .create_event(json!({ "name": "A", "slots": ["pn-09"] }))
        .await;
    let (status, body) = h.join(&eid, json!({ "nick": "   " })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_nick");

    let (status, body) = h
        .join(&eid, json!({ "nick": "Ala", "password": "x".repeat(129) }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_password");
}

#[tokio::test]
async fn serves_health_and_openapi() {
    let h = harness();
    let (status, body) = h.send(Method::GET, "/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "status": "ok" }));

    let (status, body) = h.send(Method::GET, "/openapi.json", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["paths"]["/events/{eid}/join"].is_object());
}
