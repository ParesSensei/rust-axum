use std::collections::HashMap;
use axum::{serve, Json, Router};
use axum::body::Body;
use axum::extract::{Path, Query, Request};
use axum::extract::rejection::JsonRejection;
use axum::response::Response;
use axum::routing::{get, post};
use axum_test::TestServer;
use http::{HeaderMap, Method, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, app).await.unwrap();
}

#[tokio::test]
async fn text_axum() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/").await;

    response.assert_status_ok();
    response.assert_text("Hello, world!");
}

#[tokio::test]
async fn test_method_routing() {
    async fn hello_world() -> String {
        "Hello, world!".to_string()
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello, world!");

    let response = server.post("/post").await;
    response.assert_status_ok();
    response.assert_text("Hello, world!");
}

#[tokio::test]
async fn test_request() {
    async fn hello_world(request: Request) -> String {
        format!("Hello {}", request.method())
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.post("/post").await;
    response.assert_status_ok();
    response.assert_text("Hello POST");
}

#[tokio::test]
async fn test_uri() {
    async fn hello_world(uri: Uri, method: Method) -> String {
        format!("Hello {} {}",method, uri )
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello GET http://localhost/get");

    let response = server.post("/post").await;
    response.assert_status_ok();
    response.assert_text("Hello POST http://localhost/post");
}

#[tokio::test]
async fn test_query() {
    async fn hello_world(Query(params) : Query<HashMap<String, String>>) -> String {
        let name = params.get("name").unwrap();
        format!("Hello {}",name )
    }

    let app = Router::new()
        .route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_query_param("name", "Eko").await;
    response.assert_status_ok();
    response.assert_text("Hello Eko");
}

#[tokio::test]
async fn test_header() {
    async fn hello_world(headers: HeaderMap) -> String {
        let name = headers["name"].to_str().unwrap();
        format!("Hello {}",name )
    }

    let app = Router::new()
        .route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_header("name", "Eko").await;
    response.assert_status_ok();
    response.assert_text("Hello Eko");
}

#[tokio::test]
async fn test_path_parameter() {
    async fn hello_world(Path((id, id_category)): Path<(String, String)>) -> String {
        format!("Product {}, Category {}", id, id_category )
    }

    let app = Router::new()
        .route("/products/{id}/categories/{id_category}", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/products/1/categories/2").await;
    response.assert_status_ok();
    response.assert_text("Product 1, Category 2");
}

#[tokio::test]
async fn test_body_string() {
    async fn hello_world(body: String) -> String {
        format!("Body {}", body )
    }

    let app = Router::new()
        .route("/post", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/post").text("This is body").await;
    response.assert_status_ok();
    response.assert_text("Body This is body");
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[tokio::test]
async fn test_body_json() {
    async fn hello_world(Json(request) : Json<LoginRequest>) -> String {
        format!("Hello {}", request.username )
    }

    let app = Router::new()
        .route("/post", get(hello_world));

    let request = LoginRequest{
        username: "Ekotaro".to_string(),
        password: "Password".to_string(),
    };

    let server = TestServer::new(app).unwrap();
    let response = server.get("/post").json(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Ekotaro");
}

#[tokio::test]
async fn test_json_error() {
    async fn hello_world(payload: Result<Json<LoginRequest>, JsonRejection>) -> String {
        match payload {
            Ok(request) => {
                format!("Hello {}", request.username )
            }
            Err(error) => {
                format!("Error: {:?}", error)
            }
        }
    }

    let app = Router::new()
        .route("/post", get(hello_world));

    let request = LoginRequest{
        username: "Ekotaro".to_string(),
        password: "Password".to_string(),
    };

    let server = TestServer::new(app).unwrap();
    let response = server.get("/post").json(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Ekotaro");
}

#[tokio::test]
async fn test_response() {
    async fn hello_world(request: Request) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("X-owner", "Ekotaro")
            .body(Body::from(format!("Hello {}", request.method())))
            .unwrap()
    }

    let app = Router::new()
        .route("/get", get(hello_world));


    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");
    response.assert_header("X-owner", "Ekotaro");
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
}

#[tokio::test]
async fn test_response_json() {
    async fn hello_world(request: Request) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("X-owner", "Ekotaro")
            .body(Body::from(format!("Hello {}", request.method())))
            .unwrap()
    }

    let app = Router::new()
        .route("/get", get(hello_world));


    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");
    response.assert_header("X-owner", "Ekotaro");
}