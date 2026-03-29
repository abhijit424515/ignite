mod common;
mod db;
mod embedding;
mod memory;

use actix_web::{App, HttpResponse, HttpServer, delete, get, post, put, web};
use serde::{Deserialize, Serialize};

use crate::common::error::AppError;
use crate::db::surreal::Database;
use crate::memory::model::{Memory, MemoryId};

#[derive(Deserialize, Serialize)]
struct MemoryPayload {
    content: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn add_edge(database: web::Data<Database>, path: web::Path<(String, String)>) -> Result<HttpResponse, AppError> {
    let (from_id, to_id) = path.into_inner();
    memory::service::add_edge(database.get_ref(), MemoryId(from_id), MemoryId(to_id)).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn remove_edge(database: web::Data<Database>, path: web::Path<(String, String)>) -> Result<HttpResponse, AppError> {
    let (from_id, to_id) = path.into_inner();
    memory::service::remove_edge(database.get_ref(), MemoryId(from_id), MemoryId(to_id)).await?;
    Ok(HttpResponse::NoContent().finish())
}

fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(get_memory)
        .service(create_memory)
        .service(update_memory)
        .service(delete_memory)
        .service(search_memory)
        .route("/memory/{id}/edge/{target_id}", web::post().to(add_edge))
        .route("/memory/{id}/edge/{target_id}", web::delete().to(remove_edge));
}

#[get("/memory/{id}")]
async fn get_memory(database: web::Data<Database>, id: web::Path<String>) -> Result<HttpResponse, AppError> {
    let memory = memory::service::get_memory(database.get_ref(), MemoryId(id.into_inner())).await?;
    Ok(HttpResponse::Ok().json(memory))
}

#[post("/memory")]
async fn create_memory(
    database: web::Data<Database>,
    payload: web::Json<MemoryPayload>,
) -> Result<HttpResponse, AppError> {
    let memory = memory::service::create_memory(database.get_ref(), payload.into_inner().content).await?;
    Ok(HttpResponse::Created().json(memory))
}

#[put("/memory/{id}")]
async fn update_memory(
    database: web::Data<Database>,
    id: web::Path<String>,
    payload: web::Json<MemoryPayload>,
) -> Result<HttpResponse, AppError> {
    let memory = memory::service::update_memory(
        database.get_ref(),
        MemoryId(id.into_inner()),
        payload.into_inner().content,
    )
    .await?;

    Ok(HttpResponse::Ok().json(memory))
}

#[delete("/memory/{id}")]
async fn delete_memory(database: web::Data<Database>, id: web::Path<String>) -> Result<HttpResponse, AppError> {
    memory::service::delete_memory(database.get_ref(), MemoryId(id.into_inner())).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/search")]
async fn search_memory(
    database: web::Data<Database>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, AppError> {
    let memories = memory::service::search_memory(database.get_ref(), &query.q).await?;
    Ok(HttpResponse::Ok().json(memories))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database = web::Data::new(Database::new().await.map_err(std::io::Error::other)?);

    HttpServer::new(move || {
        App::new()
            .app_data(database.clone())
            .configure(configure_app)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, http::StatusCode, test};

    #[actix_web::test]
    async fn edge_routes_work() {
        let database = web::Data::new(Database::new().await.expect("database should initialize"));
        let app = test::init_service(App::new().app_data(database.clone()).configure(configure_app)).await;

        let create_first = test::TestRequest::post()
            .uri("/memory")
            .set_json(&MemoryPayload {
                content: "first node".to_string(),
            })
            .to_request();
        let first: Memory = test::call_and_read_body_json(&app, create_first).await;

        let create_second = test::TestRequest::post()
            .uri("/memory")
            .set_json(&MemoryPayload {
                content: "second node".to_string(),
            })
            .to_request();
        let second: Memory = test::call_and_read_body_json(&app, create_second).await;

        let add_request = test::TestRequest::post()
            .uri(&format!("/memory/{}/edge/{}", first.id.0, second.id.0))
            .to_request();
        let add_response = test::call_service(&app, add_request).await;
        assert_eq!(add_response.status(), StatusCode::NO_CONTENT);

        let remove_request = test::TestRequest::delete()
            .uri(&format!("/memory/{}/edge/{}", first.id.0, second.id.0))
            .to_request();
        let remove_response = test::call_service(&app, remove_request).await;
        assert_eq!(remove_response.status(), StatusCode::NO_CONTENT);

        let get_request = test::TestRequest::get()
            .uri(&format!("/memory/{}", first.id.0))
            .to_request();
        let stored: Memory = test::call_and_read_body_json(&app, get_request).await;
        assert_eq!(stored.id, first.id);
    }

    #[actix_web::test]
    async fn create_memory_rejects_empty_content() {
        let database = web::Data::new(Database::new().await.expect("database should initialize"));
        let app = test::init_service(App::new().app_data(database).configure(configure_app)).await;

        let request = test::TestRequest::post()
            .uri("/memory")
            .set_json(&MemoryPayload {
                content: "   ".to_string(),
            })
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
