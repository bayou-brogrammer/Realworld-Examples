use crate::error::AppResult;
use crate::AppState;
use actix::Message;
use actix_web::web::{self};
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

// ================================== Client Messages ================================== //
#[derive(Debug, Message)]
#[rtype(result = "AppResult<TagsResponse>")]
pub struct GetTags;

// ================================== JSON response objects ================================== //

#[derive(Debug, Serialize)]
pub struct TagsResponse {
    pub tags: Vec<String>,
}

// ================================== Handlers ================================== //

pub async fn get_tags(state: web::Data<AppState>) -> AppResult<HttpResponse> {
    Ok(state.db.send(GetTags).await.map(|res| match res {
        Err(e) => e.error_response(),
        Ok(res) => HttpResponse::Ok().json(res),
    })?)
}
