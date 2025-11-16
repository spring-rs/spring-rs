use spring::app::AppBuilder;
use spring::plugin::ComponentRegistry;
use spring_web::{Router, WebConfigurator};

#[tokio::test]
async fn test_router_registration() {
    let mut app = AppBuilder::default();
    
    // Create a simple router
    let router = Router::new();
    app.add_router(router);
    
    // Verify router component is registered
    assert!(app.has_component::<spring_web::Routers>());
}

#[tokio::test]
async fn test_multiple_routers() {
    let mut app = AppBuilder::default();
    
    let router1 = Router::new();
    let router2 = Router::new();
    
    app.add_router(router1);
    app.add_router(router2);
    
    let routers = app.get_component::<spring_web::Routers>();
    assert!(routers.is_some());
    assert_eq!(routers.unwrap().len(), 2);
}

// Test basic axum functionality
#[tokio::test]
async fn test_simple_handler() {
    use spring_web::axum::http::{Request, StatusCode};
    use spring_web::axum::routing::get;
    use tower::ServiceExt;
    
    async fn simple_handler() -> &'static str {
        "test"
    }
    
    let app = spring_web::axum::Router::new()
        .route("/", get(simple_handler));
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(String::new())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_404_not_found() {
    use spring_web::axum::http::{Request, StatusCode};
    use spring_web::axum::routing::get;
    use tower::ServiceExt;
    
    async fn handler() -> &'static str {
        "hello"
    }
    
    let app = spring_web::axum::Router::new()
        .route("/hello", get(handler));
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(String::new())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_json_response() {
    use spring_web::axum::http::{Request, StatusCode};
    use spring_web::axum::routing::get;
    use spring_web::axum::Json;
    use tower::ServiceExt;
    
    async fn json_handler() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "message": "test",
            "status": "ok"
        }))
    }
    
    let app = spring_web::axum::Router::new()
        .route("/json", get(json_handler));
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/json")
                .body(String::new())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_with_extension() {
    use spring_web::axum::http::{Request, StatusCode};
    use spring_web::axum::routing::get;
    use spring_web::axum::Extension;
    use tower::ServiceExt;
    
    #[derive(Clone)]
    struct TestState {
        value: i32,
    }

    async fn with_state(Extension(state): Extension<TestState>) -> String {
        format!("State value: {}", state.value)
    }
    
    let state = TestState { value: 42 };
    let app = spring_web::axum::Router::new()
        .route("/state", get(with_state))
        .layer(Extension(state));
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/state")
                .body(String::new())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_path_extractor() {
    use spring_web::axum::extract::Path;
    use spring_web::axum::http::{Request, StatusCode};
    use spring_web::axum::routing::get;
    use tower::ServiceExt;
    
    async fn get_user(Path(id): Path<u32>) -> String {
        format!("User ID: {}", id)
    }
    
    // Axum 0.8 uses {param} syntax instead of :param
    let app = spring_web::axum::Router::new()
        .route("/users/{id}", get(get_user));
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/users/123")
                .body(String::new())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

mod test_error_handling {
    use spring_web::error::{KnownWebError, WebError};
    
    #[test]
    fn test_known_web_error_creation() {
        let error = KnownWebError::bad_request("Invalid input");
        assert!(error.to_string().contains("Invalid input"));
        assert!(error.to_string().contains("400"));
    }
    
    #[test]
    fn test_known_web_error_not_found() {
        let error = KnownWebError::not_found("Resource not found");
        assert!(error.to_string().contains("Resource not found"));
        assert!(error.to_string().contains("404"));
    }
    
    #[test]
    fn test_known_web_error_internal_server_error() {
        let error = KnownWebError::internal_server_error("Server error");
        assert!(error.to_string().contains("Server error"));
        assert!(error.to_string().contains("500"));
    }
    
    #[test]
    fn test_web_error_from_known_error() {
        let known_error = KnownWebError::bad_request("test");
        let web_error: WebError = known_error.into();
        
        match web_error {
            WebError::ResponseStatusError(_) => {
                // Expected
            }
            _ => panic!("Expected ResponseStatusError"),
        }
    }
}

#[tokio::test]
async fn test_router_merge() {
    let router1 = Router::new();
    let router2 = Router::new();
    
    // Test that routers can be merged
    let _merged = router1.merge(router2);
}

#[test]
fn test_web_configurator() {
    let mut app = AppBuilder::default();
    
    // Test basic configurator functionality
    let router = Router::new();
    app.add_router(router);
    
    assert!(app.has_component::<spring_web::Routers>());
}
