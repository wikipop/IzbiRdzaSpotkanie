//! Tests for token issuing, verification and the `AuthUser` extractor.

use std::sync::Arc;
use std::time::Duration;

use auth::{
    AuthError, AuthUser, Claims, DEFAULT_ISSUER, SlotAuth, TokenIssuer, TokenVerifier, authorize,
    generate_keypair,
};
use axum::Router;
use axum::body::Body;
use axum::extract::FromRef;
use axum::http::{Request, StatusCode};
use axum::routing::put;
use domain::{EventId, SlotBitmap, SlotId, UserId};
use http_body_util::BodyExt;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use tower::ServiceExt;

const TTL: Duration = Duration::from_secs(14 * 24 * 3600);

struct Fixture {
    private_pem: String,
    issuer: TokenIssuer,
    verifier: TokenVerifier,
    event_id: EventId,
    user_id: UserId,
    slots: SlotBitmap,
}

fn slot(s: &str) -> SlotId {
    s.parse().unwrap()
}

fn fixture() -> Fixture {
    let pair = generate_keypair();
    let issuer = TokenIssuer::new("k1", &pair.private_pem, DEFAULT_ISSUER, TTL).unwrap();
    let verifier =
        TokenVerifier::new(&[("k1".to_owned(), pair.public_pem)], DEFAULT_ISSUER).unwrap();
    Fixture {
        private_pem: pair.private_pem,
        issuer,
        verifier,
        event_id: EventId::generate(),
        user_id: UserId::generate(),
        slots: [slot("pn-09"), slot("pn-10")].into_iter().collect(),
    }
}

impl Fixture {
    fn token(&self) -> String {
        self.issuer
            .issue(&self.user_id, "Wiktor", &self.event_id, &self.slots)
    }

    fn claims(&self) -> Claims {
        self.verifier.verify(&self.token()).unwrap()
    }

    /// Signs arbitrary claims with the fixture's private key.
    fn sign(&self, claims: &Claims, kid: &str) -> String {
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(kid.to_owned());
        let key = EncodingKey::from_ed_pem(self.private_pem.as_bytes()).unwrap();
        jsonwebtoken::encode(&header, claims, &key).unwrap()
    }
}

#[test]
fn valid_token_passes_and_carries_claims() {
    let f = fixture();
    let claims = f.claims();
    assert_eq!(claims.iss, DEFAULT_ISSUER);
    assert_eq!(claims.sub, f.user_id.as_str());
    assert_eq!(claims.nick, "Wiktor");
    assert_eq!(claims.eid, f.event_id.as_str());
    assert_eq!(claims.exp - claims.iat, TTL.as_secs());

    let user = authorize(claims, f.event_id.as_str(), slot("pn-09")).unwrap();
    assert_eq!(user.user_id, f.user_id.as_str());
}

#[test]
fn rejects_token_signed_with_other_key() {
    let f = fixture();
    let other = generate_keypair();
    let forged = TokenIssuer::new("k1", &other.private_pem, DEFAULT_ISSUER, TTL)
        .unwrap()
        .issue(&f.user_id, "Wiktor", &f.event_id, &f.slots);
    assert!(matches!(
        f.verifier.verify(&forged),
        Err(AuthError::InvalidToken(_))
    ));
}

#[test]
fn rejects_tampered_payload() {
    let f = fixture();
    let token = f.token();
    let mut parts: Vec<&str> = token.split('.').collect();
    let mut claims = f.claims();
    claims.sub = UserId::generate().to_string();
    let payload = base64_url(&serde_json::to_vec(&claims).unwrap());
    parts[1] = &payload;
    assert!(matches!(
        f.verifier.verify(&parts.join(".")),
        Err(AuthError::InvalidToken(_))
    ));
}

#[test]
fn rejects_algorithm_other_than_eddsa() {
    let f = fixture();
    // Classic attack: an HS256 token signed with the public key as the HMAC secret.
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some("k1".to_owned());
    let public_pem = generate_keypair().public_pem;
    let token = jsonwebtoken::encode(
        &header,
        &f.claims(),
        &EncodingKey::from_secret(public_pem.as_bytes()),
    )
    .unwrap();
    assert!(matches!(
        f.verifier.verify(&token),
        Err(AuthError::InvalidToken(_))
    ));

    // `alg: none` — an unsigned token.
    let header = base64_url(br#"{"alg":"none","typ":"JWT","kid":"k1"}"#);
    let payload = base64_url(&serde_json::to_vec(&f.claims()).unwrap());
    let unsigned = format!("{header}.{payload}.");
    assert!(matches!(
        f.verifier.verify(&unsigned),
        Err(AuthError::InvalidToken(_))
    ));
}

#[test]
fn rejects_expired_token() {
    let f = fixture();
    let mut claims = f.claims();
    claims.iat -= 30 * 24 * 3600;
    claims.exp = claims.iat + 3600;
    let token = f.sign(&claims, "k1");
    assert!(matches!(
        f.verifier.verify(&token),
        Err(AuthError::InvalidToken(_))
    ));
}

#[test]
fn rejects_wrong_issuer() {
    let f = fixture();
    let mut claims = f.claims();
    claims.iss = "evil.example".to_owned();
    assert!(matches!(
        f.verifier.verify(&f.sign(&claims, "k1")),
        Err(AuthError::InvalidToken(_))
    ));
}

#[test]
fn rejects_unknown_kid_and_missing_kid() {
    let f = fixture();
    let claims = f.claims();
    assert_eq!(
        f.verifier.verify(&f.sign(&claims, "k9")),
        Err(AuthError::UnknownKey)
    );

    let key = EncodingKey::from_ed_pem(f.private_pem.as_bytes()).unwrap();
    let no_kid = jsonwebtoken::encode(&Header::new(Algorithm::EdDSA), &claims, &key).unwrap();
    assert!(matches!(
        f.verifier.verify(&no_kid),
        Err(AuthError::InvalidToken(_))
    ));
}

#[test]
fn accepts_every_configured_key_during_rotation() {
    let old = generate_keypair();
    let new = generate_keypair();
    let spec = format!(
        "k2={},k1={}",
        new.public_pem.replace('\n', "\\n"),
        old.public_pem.replace('\n', "\\n"),
    );
    let verifier = TokenVerifier::from_env_spec(&spec, DEFAULT_ISSUER).unwrap();

    let f = fixture();
    for (kid, pair) in [("k1", &old), ("k2", &new)] {
        let token = TokenIssuer::new(kid, &pair.private_pem, DEFAULT_ISSUER, TTL)
            .unwrap()
            .issue(&f.user_id, "Wiktor", &f.event_id, &f.slots);
        assert!(verifier.verify(&token).is_ok(), "token signed with {kid}");
    }
}

#[test]
fn rejects_empty_key_list() {
    assert!(TokenVerifier::new(&[], DEFAULT_ISSUER).is_err());
}

#[test]
fn rejects_event_mismatch() {
    let f = fixture();
    let other_event = EventId::generate();
    assert_eq!(
        authorize(f.claims(), other_event.as_str(), slot("pn-09")),
        Err(AuthError::EventMismatch)
    );
}

#[test]
fn rejects_slot_outside_bitmap() {
    let f = fixture();
    assert_eq!(
        authorize(f.claims(), f.event_id.as_str(), slot("pn-11")),
        Err(AuthError::SlotOutOfRange("pn-11".to_owned()))
    );
}

// --- extractor on a real router ---

#[derive(Clone)]
struct TestState {
    auth: SlotAuth,
}

impl FromRef<TestState> for SlotAuth {
    fn from_ref(state: &TestState) -> Self {
        state.auth.clone()
    }
}

fn router(f: Fixture, slot_id: &str) -> (Router, String, String) {
    let token = f.token();
    let eid = f.event_id.to_string();
    let state = TestState {
        auth: SlotAuth {
            verifier: Arc::new(f.verifier),
            slot: slot(slot_id),
        },
    };
    let app = Router::new()
        .route(
            "/events/{eid}/me",
            put(|user: AuthUser| async move { user.user_id }),
        )
        .with_state(state);
    (app, token, eid)
}

async fn call(app: Router, eid: &str, auth: Option<&str>) -> (StatusCode, serde_json::Value) {
    let mut request = Request::put(format!("/events/{eid}/me"));
    if let Some(value) = auth {
        request = request.header("Authorization", value);
    }
    let response = app
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn extractor_accepts_valid_token() {
    let f = fixture();
    let user_id = f.user_id.to_string();
    let (app, token, eid) = router(f, "pn-09");
    let response = app
        .oneshot(
            Request::put(format!("/events/{eid}/me"))
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body, user_id.as_bytes());
}

#[tokio::test]
async fn extractor_maps_errors_to_common_format() {
    let f = fixture();
    let (app, token, eid) = router(f, "pn-09");

    let (status, body) = call(app.clone(), &eid, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "missing_token");

    let (status, body) = call(app.clone(), &eid, Some("Bearer garbage")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");

    let other = EventId::generate().to_string();
    let (status, body) = call(app, &other, Some(&format!("Bearer {token}"))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "event_mismatch");
}

#[tokio::test]
async fn extractor_rejects_slot_outside_event() {
    let f = fixture();
    let (app, token, eid) = router(f, "nd-23");
    let (status, body) = call(app, &eid, Some(&format!("Bearer {token}"))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "slot_out_of_range");
}

fn base64_url(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
