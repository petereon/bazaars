use std::{env, io::Read, sync::Arc};

use axum::{
    async_trait,
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use axum_typed_multipart::TypedMultipart;
use bazaars::{
    db,
    models::ad::{Ad, AdContent, AdRequest},
    repos::{
        ad_repo::{AdFilter, AdRepo, PostgresAdRepo},
        image_repo::{ImageRepo, LocalImageRepo},
    },
};

#[derive(Clone)]
struct AppState {
    ad_repo: Arc<dyn AdRepo>,
    image_repo: Arc<dyn ImageRepo>,
}

#[tokio::main]
async fn main() {
    let db_manager = db::DbManager::new(
        env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set")
            .as_str(),
    );

    let ad_repo = PostgresAdRepo::new(db_manager);
    let image_repo = LocalImageRepo::new("images".to_string());

    let app: Router = Router::new()
        .route("/ads", get(get_ads))
        .route("/ads/:id", get(get_ad))
        .route("/images/:id", get(get_image))
        .route("/ads", post(create_ad))
        .route("/ads/:id", put(update_ad))
        .route("/ads/:id", delete(delete_ad))
        .with_state(AppState {
            ad_repo,
            image_repo,
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// #[derive(serde::Serialize)]
// struct CursorRes<T> {
//     cursor: String,
//     items: Vec<T>,
// }

// #[derive(serde::Deserialize)]
// struct CursorReq {
//     cursor: Option<String>,
//     count: u8,
//     filters: AdFilter,
// }

#[derive(serde::Serialize)]
struct PaginatedRes<T> {
    page: u32,
    items: Vec<T>,
}

#[derive(serde::Deserialize, Clone)]
struct PaginatedReq {
    per_page: Option<u32>,
    offset: Option<u32>,
    filters: Option<AdFilter>,
}

async fn get_ads(
    State(state): State<AppState>,
    payload: Option<Json<PaginatedReq>>,
) -> Result<Json<PaginatedRes<Ad>>, StatusCode> {
    // Stub implementation
    let params = match payload {
        Some(Json(payload)) => payload,
        None => PaginatedReq {
            per_page: Some(10),
            offset: Some(0),
            filters: None,
        },
    };
    let per_page = params.per_page.unwrap_or(10);

    let offset = params.offset.unwrap_or(0);

    match state
        .ad_repo
        .get_page(offset, per_page, params.filters.unwrap_or_default())
        .await
    {
        Ok(items) => Ok(Json(PaginatedRes {
            items,
            page: offset / per_page + 1,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_image(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.image_repo.get_image(&id).await {
        Ok(image) => {
            let content_type = image.mime_type;
            let bytes = image.bytes;
            let body = Body::from(bytes);
            let response = axum::http::Response::builder()
                .header("Content-Type", content_type)
                .body(body)
                .unwrap();
            Ok(response)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
async fn get_ad(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Ad>, StatusCode> {
    let id = id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.ad_repo.get_by_id(id).await {
        Ok(Some(ad)) => Ok(Json(ad)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
async fn create_ad(
    State(state): State<AppState>,
    TypedMultipart(payload): TypedMultipart<AdRequest>,
) -> Result<String, StatusCode> {
    let ad = AdContent {
        title: payload.title,
        description: payload.description,
        price: payload.price,
        user_email: payload.user_email,
        user_phone: payload.user_phone,
        top_ad: payload.top_ad,
    };

    let mut image_ids = Vec::new();

    for image in payload.images {
        let image_data: Vec<u8> = image.contents.bytes().filter_map(Result::ok).collect();
        let image_id = state
            .image_repo
            .create_image(
                image.metadata.file_name.unwrap(),
                image_data,
                image.metadata.content_type.unwrap(),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        image_ids.push(image_id);
    }

    let ad = state
        .ad_repo
        .create(ad, image_ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(ad.id.to_string())
}

async fn update_ad(
    Path(id): Path<String>,
    State(state): State<AppState>,
    TypedMultipart(payload): TypedMultipart<AdRequest>,
) -> StatusCode {
    // Stub implementation
    StatusCode::OK
}

async fn delete_ad(Path(id): Path<String>, State(state): State<AppState>) -> StatusCode {
    // Stub implementation
    StatusCode::NO_CONTENT
}
