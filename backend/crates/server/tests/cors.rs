use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::routing::get;
use server::{OriginPattern, cors_layer, parse_origins};
use tower::ServiceExt;

#[test]
fn parses_exact_and_any_port_origins() {
    let patterns = parse_origins("https://meet.izbi.beer/, http://localhost:*").unwrap();
    assert_eq!(
        patterns,
        [
            OriginPattern::Exact("https://meet.izbi.beer".to_owned()),
            OriginPattern::AnyPort("http://localhost:".to_owned()),
        ]
    );
}

#[test]
fn rejects_empty_origin_list() {
    assert!(parse_origins(" , ").is_err());
}

#[test]
fn matches_origins() {
    let patterns = parse_origins("https://meet.izbi.beer,http://localhost:*").unwrap();
    let allowed = |origin: &str| patterns.iter().any(|p| p.matches(origin));
    assert!(allowed("https://meet.izbi.beer"));
    assert!(allowed("http://localhost:5173"));
    assert!(!allowed("http://localhost:"));
    assert!(!allowed("http://localhost:5173.evil.com"));
    assert!(!allowed("https://meet.izbi.beer.evil.com"));
    assert!(!allowed("http://meet.izbi.beer"));
}

#[tokio::test]
async fn answers_preflight_for_allowed_origin_only() {
    let app = Router::new()
        .route("/", get(|| async {}))
        .layer(cors_layer(parse_origins("http://localhost:*").unwrap()));
    let preflight = |origin: &str| {
        Request::builder()
            .method(Method::OPTIONS)
            .uri("/")
            .header("Origin", origin)
            .header("Access-Control-Request-Method", "PUT")
            .header(
                "Access-Control-Request-Headers",
                "authorization,content-type",
            )
            .body(Body::empty())
            .unwrap()
    };

    let response = app
        .clone()
        .oneshot(preflight("http://localhost:3000"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(
        headers["access-control-allow-origin"],
        "http://localhost:3000"
    );
    assert_eq!(headers["access-control-max-age"], "7200");

    let response = app
        .oneshot(preflight("https://evil.example"))
        .await
        .unwrap();
    assert!(
        !response
            .headers()
            .contains_key("access-control-allow-origin")
    );
}
