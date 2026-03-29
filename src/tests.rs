use actix_web::{App, http::StatusCode, test, web};
use serde_json::json;

use crate::api::{MemoryPayload, configure};
use crate::common::error::AppError;
use crate::db::surreal::Database;
use crate::memory::model::{EdgeData, Memory, MemoryId};
use crate::memory::service;

#[actix_web::test]
async fn edge_routes_work() {
    let database = web::Data::new(Database::new().await.expect("database should initialize"));
    let app = test::init_service(App::new().app_data(database.clone()).configure(configure)).await;

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
        .set_json(json!({
            "content": "parent",
            "weight": 2,
            "kind": "reference"
        }))
        .to_request();
    let add_response = test::call_service(&app, add_request).await;
    assert_eq!(add_response.status(), StatusCode::NO_CONTENT);

    let list_request = test::TestRequest::get()
        .uri(&format!("/memory/{}/edge", first.id.0))
        .to_request();
    let edges: Vec<MemoryId> = test::call_and_read_body_json(&app, list_request).await;
    assert_eq!(edges, vec![second.id.clone()]);

    let remove_request = test::TestRequest::delete()
        .uri(&format!("/memory/{}/edge/{}", first.id.0, second.id.0))
        .to_request();
    let remove_response = test::call_service(&app, remove_request).await;
    assert_eq!(remove_response.status(), StatusCode::NO_CONTENT);

    let list_request = test::TestRequest::get()
        .uri(&format!("/memory/{}/edge", first.id.0))
        .to_request();
    let edges: Vec<MemoryId> = test::call_and_read_body_json(&app, list_request).await;
    assert!(edges.is_empty());

    let get_request = test::TestRequest::get()
        .uri(&format!("/memory/{}", first.id.0))
        .to_request();
    let stored: Memory = test::call_and_read_body_json(&app, get_request).await;
    assert_eq!(stored.id, first.id);
}

#[actix_web::test]
async fn create_memory_rejects_empty_content() {
    let database = web::Data::new(Database::new().await.expect("database should initialize"));
    let app = test::init_service(App::new().app_data(database).configure(configure)).await;

    let request = test::TestRequest::post()
        .uri("/memory")
        .set_json(&MemoryPayload {
            content: "   ".to_string(),
        })
        .to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn memory_flow_works() {
    let database = Database::new().await.expect("database should initialize");

    let first = service::create_memory(&database, "rust web memory".to_string())
        .await
        .expect("memory should be created");
    let second = service::create_memory(&database, "surreal graph node".to_string())
        .await
        .expect("memory should be created");

    let loaded = service::get_memory(&database, first.id.clone())
        .await
        .expect("memory should load");
    assert_eq!(loaded.content, "rust web memory");

    let updated =
        service::update_memory(&database, first.id.clone(), "rust actix memory".to_string())
            .await
            .expect("memory should update");
    assert_eq!(updated.content, "rust actix memory");

    let search_results = service::search_memory(&database, "actix")
        .await
        .expect("search should work");
    assert!(search_results.iter().any(|memory| memory.id == first.id));

    service::add_edge(
        &database,
        first.id.clone(),
        second.id.clone(),
        EdgeData {
            content: "related".to_string(),
            extra: Default::default(),
        },
    )
        .await
        .expect("edge should be added");
    let edges = service::list_edges(&database, first.id.clone())
        .await
        .expect("edges should load");
    assert_eq!(edges, vec![second.id.clone()]);

    service::remove_edge(&database, first.id.clone(), second.id.clone())
        .await
        .expect("edge should be removed");
    let edges = service::list_edges(&database, first.id.clone())
        .await
        .expect("edges should load");
    assert!(edges.is_empty());

    service::delete_memory(&database, second.id.clone())
        .await
        .expect("memory should delete");

    let error = service::get_memory(&database, second.id.clone())
        .await
        .expect_err("deleted memory should be missing");
    assert!(matches!(error, AppError::NotFound));
}

#[actix_web::test]
async fn add_edge_rejects_self_links() {
    let database = Database::new().await.expect("database should initialize");
    let memory = service::create_memory(&database, "self edge check".to_string())
        .await
        .expect("memory should be created");

    let error = service::add_edge(
        &database,
        memory.id.clone(),
        memory.id.clone(),
        EdgeData {
            content: "self".to_string(),
            extra: Default::default(),
        },
    )
        .await
        .expect_err("self edge should fail");

    assert!(matches!(error, AppError::BadRequest(_)));
}

#[actix_web::test]
async fn add_edge_rejects_blank_content() {
    let database = web::Data::new(Database::new().await.expect("database should initialize"));
    let app = test::init_service(App::new().app_data(database).configure(configure)).await;

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

    let request = test::TestRequest::post()
        .uri(&format!("/memory/{}/edge/{}", first.id.0, second.id.0))
        .set_json(json!({ "content": "   " }))
        .to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
