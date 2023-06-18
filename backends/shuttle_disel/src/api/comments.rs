use crate::error::AppResult;
use crate::utils::{authenticate, Auth, CustomDateTime};
use crate::AppState;
use actix::Message;
use actix_web::web::{self, Json};
use actix_web::{HttpRequest, HttpResponse, ResponseError};
use futures::{FutureExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::profile::ProfileResponseInner;

#[derive(Debug, Deserialize)]
pub struct In<T> {
    comment: T,
}

// ================================== Extractors ================================== //
// ================================== Client Messages ================================== //

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AddCommentData {
    #[validate(length(min = 1, message = "body can't be blank"))]
    pub body: String,
}

#[derive(Debug, Message)]
#[rtype(result = "AppResult<CommentResponse>")]
pub struct AddCommentOuter {
    pub auth: Auth,
    pub slug: String,
    pub comment: AddCommentData,
}

#[derive(Debug, Message)]
#[rtype(result = "AppResult<CommentListResponse>")]
pub struct GetComments {
    pub slug: String,
    pub auth: Option<Auth>,
}

#[derive(Debug, Message)]
#[rtype(result = "AppResult<serde_json::Value>")]
pub struct DeleteComment {
    pub auth: Auth,
    pub slug: String,
    pub comment_id: i32,
}

// ================================== JSON response objects ================================== //

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub comment: CommentResponseInner,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentResponseInner {
    pub id: i32,
    pub body: String,
    pub created_at: CustomDateTime,
    pub updated_at: CustomDateTime,
    pub author: ProfileResponseInner,
}

#[derive(Debug, Serialize)]
pub struct CommentListResponse {
    pub comments: Vec<CommentResponseInner>,
}

// ================================== HANDLERS ================================== //

pub async fn add_comment(
    req: HttpRequest,
    slug: web::Path<String>,
    state: web::Data<AppState>,
    form: Json<In<AddCommentData>>,
) -> AppResult<HttpResponse> {
    let db = &state.db;

    let comment = form.into_inner().comment;
    comment.validate()?;

    Ok(authenticate(&state, &req)
        .and_then(|auth| async move {
            db.send(AddCommentOuter {
                auth,
                comment,
                slug: slug.into_inner(),
            })
            .await?
        })
        .map(|res| match res {
            Err(e) => e.error_response(),
            Ok(res) => HttpResponse::Ok().json(res),
        })
        .await)
}

pub async fn get_comments(
    req: HttpRequest,
    slug: web::Path<String>,
    state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let db = &state.db;

    Ok(authenticate(&state, &req)
        .then(|auth| async move {
            db.send(GetComments {
                auth: auth.ok(),
                slug: slug.into_inner(),
            })
            .await?
        })
        .map(|res| match res {
            Err(e) => e.error_response(),
            Ok(res) => HttpResponse::Ok().json(res),
        })
        .await)
}

#[derive(Debug, Deserialize)]
pub struct ArticleCommentPath {
    slug: String,
    comment_id: i32,
}

pub async fn delete_comment(
    req: HttpRequest,
    path: web::Path<ArticleCommentPath>,
    state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let db = &state.db;

    Ok(authenticate(&state, &req)
        .and_then(|auth| async move {
            db.send(DeleteComment {
                auth,
                slug: path.slug.to_owned(),
                comment_id: path.comment_id.to_owned(),
            })
            .await?
        })
        .map(|res| match res {
            Err(e) => e.error_response(),
            Ok(res) => HttpResponse::Ok().json(res),
        })
        .await)
}
