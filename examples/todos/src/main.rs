//! Provides a RESTful web server managing some Todos.
//!
//! API will be:
//!
//! - `GET /todos`: return a JSON list of Todos.
//! - `POST /todos`: create a new Todo.
//! - `PUT /todos/:id`: update a specific Todo.
//! - `DELETE /todos/:id`: delete a specific Todo.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -p example-todos
//! ```

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::{
    net::SocketAddr,
    time::Duration,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};
use tracing::{event, Level};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter};
use uuid::Uuid;

pub mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "example_todos=debug,tower_http=debug")
    }

    init_tracing()?;

    let db_url = std::env::var_os("DATABASE_URL")
        .unwrap_or_else(|| std::ffi::OsString::from("postgres://postgres@localhost:5432/todos"))
        .into_string()
        .map_err(|_| anyhow::anyhow!("DATABASE_URL is malformed"))?;

    let pool = PgPool::connect(db_url.as_str()).await?;

    // Compose the routes
    let app = Router::new()
        .route("/todos", get(todos_index).post(todos_create))
        .route("/todos/:id", patch(todos_update).delete(todos_delete))
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .layer(AddExtensionLayer::new(pool))
                .into_inner(),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

// The query parameters for todos index
#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    pub offset: Option<i64>, // FIXME: should be unsigned
    pub limit: Option<i64>,  // FIXME: should be unsigned
}

async fn todos_index(
    pagination: Option<Query<Pagination>>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let Query(pagination) = pagination.unwrap_or_default();

    event!(Level::INFO, "GET /todos");
    let todos = db::find_all_todos(pool, pagination)
        .await
        .expect("`todo` table query failed"); // FIXME: use error result

    Json(todos)
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    text: String,
}

async fn todos_create(
    Json(input): Json<CreateTodo>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let todo = Todo {
        id: Uuid::new_v4(),
        text: input.text,
        completed: false,
    };

    db::insert_todo(pool, todo.clone())
        .await
        .expect("`todo` table insert failed"); // FIXME: use error result

    (StatusCode::CREATED, Json(todo))
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

async fn todos_update(
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTodo>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut todo = db::find_one_todo(&pool, id)
        .await
        .expect("FIXME: ")
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(text) = input.text {
        todo.text = text;
    }

    if let Some(completed) = input.completed {
        todo.completed = completed;
    }

    let todo = db::update_todo(pool, todo)
        .await
        .expect("FIXME: ");

    Ok(Json(todo))
}

async fn todos_delete(Path(id): Path<Uuid>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let deleted = db::delete_todo(pool, id)
        .await
        .expect("`todo` table delete failed"); // FIXME: use error result

    if deleted.is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

fn init_tracing() -> anyhow::Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::CLOSE);

    let jaeger_tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("todo-service")
        .install_simple()?;

    let opentelemetry_layer = tracing_opentelemetry::layer()
        .with_tracer(jaeger_tracer);

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap()
        .add_directive("hyper=info".parse()?);

    let subscriber = tracing_subscriber::Registry::default()
        .with(fmt_layer)
        .with(opentelemetry_layer)
        .with(filter_layer);

     tracing::subscriber::set_global_default(subscriber)?;

     Ok(())
}

#[derive(Debug, Serialize, Clone)]
pub struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}
