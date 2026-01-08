//! Integration tests for the chain parser API.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use vulcan_chain_parser_api::{build_router, create_app_state};

/// Create a test router with a real database connection.
///
/// Requires DATABASE_URL to be set.
fn create_test_app() -> axum::Router {
    dotenvy::dotenv().ok();
    let conn = vulcan_core::establish_connection();
    let state = create_app_state(conn);
    build_router(state)
}

/// Helper to read response body as JSON.
async fn body_to_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_parse_valid_workflow() {
    let app = create_test_app();

    let workflow_content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"
    fragment { run "npm build" }
    fragment { run "npm test" }
}
"#;

    let request_body = json!({
        "content": workflow_content,
        "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
        "source_file_path": ".vulcan/ci.kdl",
        "repository_url": "https://github.com/test/repo",
        "branch": "main",
        "trigger": "push"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response.into_body()).await;
    assert!(body.get("chain_id").is_some());
    assert_eq!(body["fragment_count"], 2);
    assert_eq!(body["message"], "Workflow parsed and stored successfully");
}

#[tokio::test]
async fn test_parse_invalid_kdl_syntax() {
    let app = create_test_app();

    let request_body = json!({
        "content": "this is not valid kdl {{{",
        "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_to_json(response.into_body()).await;
    assert_eq!(body["code"], "PARSE_ERROR");
    assert!(body["error"].as_str().unwrap().contains("parse error"));
}

#[tokio::test]
async fn test_parse_missing_required_field() {
    let app = create_test_app();

    // Missing 'machine' in chain
    let workflow_content = r#"
version "0.1"
triggers "push"

chain {
    fragment { run "npm build" }
}
"#;

    let request_body = json!({
        "content": workflow_content,
        "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_to_json(response.into_body()).await;
    assert_eq!(body["code"], "PARSE_ERROR");
    assert!(body["error"].as_str().unwrap().contains("machine"));
}

#[tokio::test]
async fn test_parse_trigger_mismatch() {
    let app = create_test_app();

    // Workflow only supports "push" but we're triggering with "pull_request"
    let workflow_content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"
    fragment { run "npm build" }
}
"#;

    let request_body = json!({
        "content": workflow_content,
        "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
        "trigger": "pull_request"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_to_json(response.into_body()).await;
    assert_eq!(body["code"], "PARSE_ERROR");
    assert!(body["error"].as_str().unwrap().contains("trigger"));
}

#[tokio::test]
async fn test_parse_invalid_trigger_type() {
    let app = create_test_app();

    let workflow_content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"
    fragment { run "npm build" }
}
"#;

    let request_body = json!({
        "content": workflow_content,
        "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
        "trigger": "invalid_trigger_type"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_to_json(response.into_body()).await;
    assert_eq!(body["code"], "INVALID_REQUEST");
}

#[tokio::test]
async fn test_parse_with_parallel_fragments() {
    let app = create_test_app();

    let workflow_content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment { run "npm install" }

    parallel {
        fragment { run "npm test:unit" }
        fragment { run "npm test:e2e" }
        fragment { run "npm lint" }
    }

    fragment { run "npm build" }
}
"#;

    let request_body = json!({
        "content": workflow_content,
        "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response.into_body()).await;
    // 1 install + 1 parallel group + 3 parallel children + 1 build = 6 fragments
    assert_eq!(body["fragment_count"], 6);
}

#[tokio::test]
async fn test_parse_with_conditions() {
    let app = create_test_app();

    let workflow_content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment { run "npm build" }

    fragment {
        condition "$BRANCH == 'main'"
        run "npm run deploy:prod"
        machine "prod-worker"
    }
}
"#;

    let request_body = json!({
        "content": workflow_content,
        "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response.into_body()).await;
    assert_eq!(body["fragment_count"], 2);
}

#[tokio::test]
async fn test_parse_invalid_json_body() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from("not valid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Invalid JSON returns 400 BAD_REQUEST
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_parse_missing_tenant_id() {
    let app = create_test_app();

    let request_body = json!({
        "content": "version \"0.1\"\ntriggers \"push\"\nchain { machine \"default\" fragment { run \"test\" } }"
        // Missing tenant_id
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Missing required field in JSON should return 422
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
