use crate::error::AppResult;
use crate::models::articles::Article;
use crate::models::user::User;
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
    article: T,
}

// ================================== Extractors ================================== //

#[derive(Debug, Deserialize)]
pub struct ArticlesParams {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub limit: Option<usize>,  // <- if not set, is 20
    pub offset: Option<usize>, // <- if not set, is 0
}

// ================================== Client Messages ================================== //

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticleData {
    #[serde(default)]
    pub tag_list: Vec<String>,
    #[validate(length(min = 1, message = "title can't be blank"))]
    pub title: String,
    #[validate(length(min = 1, message = "description can't be blank"))]
    pub description: String,
    #[validate(length(min = 1, message = "body can't be blank"))]
    pub body: String,
}

#[derive(Debug, Message)]
#[rtype(result = "AppResult<ArticleResponse>")]
pub struct CreateArticleOuter {
    pub auth: Auth,
    pub article: CreateArticleData,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticleData {
    pub body: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tag_list: Option<Vec<String>>,
}

#[derive(Debug, Message)]
#[rtype(result = "AppResult<ArticleResponse>")]
pub struct UpdateArticleOuter {
    pub auth: Auth,
    pub slug: String,
    pub article: UpdateArticleData,
}

#[derive(Debug, Message)]
#[rtype(result = "AppResult<ArticleResponse>")]
pub struct GetArticle {
    pub slug: String,
    pub auth: Option<Auth>,
}

#[derive(Debug, Message)]
#[rtype(result = "AppResult<ArticleListResponse>")]
pub struct GetArticles {
    // auth is option in case authentication fails or isn't present
    pub auth: Option<Auth>,
    pub params: ArticlesParams,
}

#[derive(Debug, Message)]
#[rtype(result = "AppResult<serde_json::Value>")]
pub struct DeleteArticle {
    pub auth: Auth,
    pub slug: String,
}

// ================================== JSON response objects ================================== //

#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub article: ArticleResponseInner,
}

impl ArticleResponse {
    pub fn new(
        article: Article,
        author: User,
        tags: Vec<String>,
        favorited: bool,
        favorites_count: i64,
        following: bool,
    ) -> Self {
        Self {
            article: ArticleResponseInner {
                favorited,
                tag_list: tags,
                favorites_count,
                slug: article.slug,
                body: article.body,
                title: article.title,
                description: article.description,
                created_at: CustomDateTime(article.created_at),
                updated_at: CustomDateTime(article.updated_at),
                author: ProfileResponseInner {
                    following,
                    bio: author.bio,
                    image: author.image,
                    username: author.username,
                },
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleResponseInner {
    pub slug: String,
    pub body: String,
    pub title: String,
    pub favorited: bool,
    pub description: String,
    pub tag_list: Vec<String>,
    pub favorites_count: i64,
    pub created_at: CustomDateTime,
    pub updated_at: CustomDateTime,
    pub author: ProfileResponseInner,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleListResponse {
    pub articles: Vec<ArticleResponseInner>,
    pub articles_count: usize,
}

// ================================== Handlers ================================== //

#[actix_web::post("")]
async fn create_article(
    req: HttpRequest,
    state: web::Data<AppState>,
    form: Json<In<CreateArticleData>>,
) -> AppResult<HttpResponse> {
    let db = &state.db;
    let article = form.into_inner().article;
    article.validate()?;

    Ok(authenticate(&state, &req)
        .and_then(|auth| async move { db.send(CreateArticleOuter { auth, article }).await? })
        .map(|res| match res {
            Err(e) => e.error_response(),
            Ok(res) => HttpResponse::Ok().json(res),
        })
        .await)
}

#[actix_web::get("/{slug}")]
async fn get_article(
    req: HttpRequest,
    slug: web::Path<String>,
    state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let db = &state.db;

    Ok(authenticate(&state, &req)
        .then(|auth| async move {
            db.send(GetArticle {
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

#[actix_web::put("/{slug}")]
async fn update_article(
    req: HttpRequest,
    slug: web::Path<String>,
    state: web::Data<AppState>,
    form: Json<In<UpdateArticleData>>,
) -> AppResult<HttpResponse> {
    let db = &state.db;

    let article = form.into_inner().article;
    article.validate()?;

    Ok(authenticate(&state, &req)
        .and_then(|auth| async move {
            db.send(UpdateArticleOuter {
                auth,
                article,
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

#[actix_web::delete("/{slug}")]
async fn delete_article(
    req: HttpRequest,
    slug: web::Path<String>,
    state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let db = &state.db;

    Ok(authenticate(&state, &req)
        .and_then(|auth| async move {
            db.send(DeleteArticle {
                auth,
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
