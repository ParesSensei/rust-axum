use axum::body::{Body, Bytes};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Multipart, Path, Query, Request, State};
use axum::middleware::{from_fn, map_request, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{serve, Extension, Form, Json, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use axum_test::multipart::{MultipartForm, Part};
use axum_test::TestServer;
use http::{HeaderMap, HeaderValue, Method, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use axum::error_handling::HandleError;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, app).await.unwrap();
}

#[tokio::test]
async fn text_axum() {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

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
        format!("Hello {} {}", method, uri)
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
    async fn hello_world(Query(params): Query<HashMap<String, String>>) -> String {
        let name = params.get("name").unwrap();
        format!("Hello {}", name)
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_query_param("name", "Eko").await;
    response.assert_status_ok();
    response.assert_text("Hello Eko");
}

#[tokio::test]
async fn test_header() {
    async fn hello_world(headers: HeaderMap) -> String {
        let name = headers["name"].to_str().unwrap();
        format!("Hello {}", name)
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_header("name", "Eko").await;
    response.assert_status_ok();
    response.assert_text("Hello Eko");
}

#[tokio::test]
async fn test_path_parameter() {
    async fn hello_world(Path((id, id_category)): Path<(String, String)>) -> String {
        format!("Product {}, Category {}", id, id_category)
    }

    let app = Router::new().route("/products/{id}/categories/{id_category}", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/products/1/categories/2").await;
    response.assert_status_ok();
    response.assert_text("Product 1, Category 2");
}

#[tokio::test]
async fn test_body_string() {
    async fn hello_world(body: String) -> String {
        format!("Body {}", body)
    }

    let app = Router::new().route("/post", get(hello_world));

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
    async fn hello_world(Json(request): Json<LoginRequest>) -> String {
        format!("Hello {}", request.username)
    }

    let app = Router::new().route("/post", get(hello_world));

    let request = LoginRequest {
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
                format!("Hello {}", request.username)
            }
            Err(error) => {
                format!("Error: {:?}", error)
            }
        }
    }

    let app = Router::new().route("/post", get(hello_world));

    let request = LoginRequest {
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

    let app = Router::new().route("/get", get(hello_world));

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
    async fn hello_world() -> Json<LoginResponse> {
        Json(LoginResponse {
            token: "Token".to_string(),
        })
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("{\"token\":\"Token\"}");
}

#[tokio::test]
async fn test_response_tuple() {
    async fn hello_world() -> (Response<()>, Json<LoginResponse>) {
        (
            Response::builder()
                .status(StatusCode::OK)
                .header("X-owner", "Ekotaro")
                .body(())
                .unwrap(),
            Json(LoginResponse {
                token: "Token".to_string(),
            }),
        )
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("{\"token\":\"Token\"}");
    response.assert_header("X-owner", "Ekotaro");
}

#[tokio::test]
async fn test_response_tuple3() {
    async fn hello_world() -> (StatusCode, HeaderMap, Json<LoginResponse>) {
        let mut headers = HeaderMap::new();
        headers.insert("X-owner", HeaderValue::from_str("Ekotaro").unwrap());

        (
            StatusCode::OK,
            headers.clone(),
            Json(LoginResponse {
                token: "Token".to_string(),
            }),
        )
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("{\"token\":\"Token\"}");
    response.assert_header("X-owner", "Ekotaro");
}

#[tokio::test]
async fn test_form() {
    async fn hello_world(Form(request): Form<LoginRequest>) -> String {
        format!("Hello {}", request.username)
    }

    let app = Router::new().route("/post", post(hello_world));

    let request = LoginRequest {
        username: "Ekotaro".to_string(),
        password: "Password".to_string(),
    };

    let server = TestServer::new(app).unwrap();
    let response = server.post("/post").form(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Ekotaro");
}

#[tokio::test]
async fn test_multipart() {
    async fn hello_world(mut payload: Multipart) -> String {
        let mut profile: Bytes = Bytes::new();
        let mut username: String = "".to_string();

        while let Some(field) = payload.next_field().await.unwrap() {
            if field.name().unwrap_or("") == "profile" {
                profile = field.bytes().await.unwrap();
            } else if field.name().unwrap_or("") == "username" {
                username = field.text().await.unwrap();
            }
        }

        assert!(profile.len() > 0);
        format!("Hello {}", username)
    }

    let app = Router::new().route("/post", post(hello_world));

    let request = MultipartForm::new()
        .add_text("username", "Ekotaro")
        .add_text("password", "Password")
        .add_part("profile", Part::bytes(Bytes::from("Contoh")));

    let server = TestServer::new(app).unwrap();
    let response = server.post("/post").multipart(request).await;
    response.assert_status_ok();
    response.assert_text("Hello Ekotaro");
}

#[tokio::test]
async fn test_cookie_response() {
    async fn hello_world(query: Query<HashMap<String, String>>) -> (CookieJar, String) {
        let name = query.get("name").unwrap();

        (
            CookieJar::new().add(Cookie::new("name", name.clone())),
            format!("Hello {}", name.clone()),
        )
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_query_param("name", "Ekotaro").await;
    response.assert_status_ok();
    response.assert_text("Hello Ekotaro");
    response.assert_header("Set-Cookie", "name=Ekotaro");
}

#[tokio::test]
async fn test_cookie_request() {
    async fn hello_world(cookie: CookieJar) -> String {
        let name = cookie.get("name").unwrap().value();

        format!("Hello {}", name)
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server
        .get("/get")
        .add_header("Cookie", "name=Ekotaro")
        .await;
    response.assert_status_ok();
    response.assert_text("Hello Ekotaro");
}

async fn log_middleware(request: Request, next: Next) -> Response {
    println!("receive request {} {}", request.method(), request.uri());
    let response = next.run(request).await;
    println!("Send response {}", response.status());
    response
}

async fn request_id_middleware<T>(mut request: Request<T>) -> Request<T> {
    let request_id = "12345";
    request
        .headers_mut()
        .insert("X-Request-Id", request_id.parse().unwrap());
    request
}

#[tokio::test]
async fn test_middleware() {
    async fn hello_world(method: Method, header_map: HeaderMap) -> String {
        println!("Execute handler");
        let request_id = header_map.get("X-Request-Id").unwrap().to_str().unwrap();
        format!("Hello {} {}", method, request_id)
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .layer(map_request(request_id_middleware))
        .layer(from_fn(log_middleware));

    let server = TestServer::new(app).unwrap();
    let response = server
        .get("/get")
        .add_header("Cookie", "name=Ekotaro")
        .await;
    response.assert_status_ok();
    response.assert_text("Hello GET 12345");
}

struct AppError {
    code: i32,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.code as u16).unwrap(),
            self.message,
        )
            .into_response()
    }
}

#[tokio::test]
async fn test_error_handling() {
    async fn hello_world(method: Method) -> Result<String, AppError> {
        if method == Method::POST {
            Ok("OK".to_string())
        } else {
            Err(AppError {
                code : 400,
                message: "Bad Request".to_string(),
            })
        }
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status(StatusCode::BAD_REQUEST);
    response.assert_text("Bad Request");

    let response = server.post("/post").await;
    response.assert_status(StatusCode::OK);
    response.assert_text("OK");
}

#[tokio::test]
async fn test_unexpected_error() {
    async fn route(request: Request) -> Result<Response, anyhow::Error> {
        if request.method() == Method::POST {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())?)
        } else {
            Err(anyhow!("Bad Request"))
        }
    }

    async fn handle_error(err: anyhow::Error) -> (StatusCode, String) {
       ( StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal Server Error: {}", err),)
    }

    let route_service = tower::service_fn(route);

    let app = Router::new()
        .route_service("/get", HandleError::new(route_service, handle_error));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    response.assert_text("Internal Server Error: Bad Request");
}

struct DatabaseConfig {
    total: i32
}

#[tokio::test]
async fn test_state_extractor() {
    let database_state = Arc::new(DatabaseConfig { total: 100 });

    async fn route(State(database): State<Arc<DatabaseConfig>>) -> String {
        format!("Total: {}", database.total)
    }

    let app = Router::new()
    .route("/get", get(route))
    .with_state(database_state);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total: 100");
}

#[tokio::test]
async fn test_state_extension() {
    let database_state = Arc::new(DatabaseConfig { total: 100 });

    async fn route(Extension(database): Extension<Arc<DatabaseConfig>>) -> String {
        format!("Total: {}", database.total)
    }

    let app = Router::new()
        .route("/get", get(route))
        .layer(Extension(database_state));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total: 100");
}

#[tokio::test]
async fn test_state_closure_capture() {
    let database_state = Arc::new(DatabaseConfig { total: 100 });

    async fn route(database: Arc<DatabaseConfig>) -> String {
        format!("Total: {}", database.total)
    }

    let app = Router::new()
        .route("/get", get({
            let database_state = Arc::clone(&database_state);
            move || route(database_state)
        }))
        .layer(Extension(database_state));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total: 100");
}

#[tokio::test]
async fn test_multiple_route() {
    async fn hello_world(method: Method) -> String {
        format!("Hello {}", method)
    }

    let first = Router::new().route("/first", get(hello_world));
    let second = Router::new().route("/second", get(hello_world));

    let app = Router::new().merge(first).merge(second);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/first").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.get("/second").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");
}

#[tokio::test]
async fn test_multiple_route_nest() {
    async fn hello_world(method: Method) -> String {
        format!("Hello {}", method)
    }

    let first = Router::new().route("/first", get(hello_world));
    let second = Router::new().route("/second", get(hello_world));

    let app = Router::new()
        .nest("/api/users", first)
        .nest("/api/products", second);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/api/users/first").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.get("/api/products/second").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");
}