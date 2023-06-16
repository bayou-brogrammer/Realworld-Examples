use crate::error::AppResult;
use crate::utils::jwt::GenerateJwt;
use crate::utils::{authenticate, Auth};
use crate::{models::user::User, AppState};
use actix::Message;
use actix_web::web::{self, Json};
use actix_web::{HttpRequest, HttpResponse, ResponseError};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct In<U> {
    user: U,
}

// ================================== UI Messages ================================== //

#[derive(Deserialize, Validate, Debug, Clone, Message)]
#[rtype(result = "AppResult<UserResponse>")]
pub struct RegistrationUser {
    #[validate(
        non_control_character(message = "user name can't contain non-ascii charactors"),
        length(min = 1, message = "user name can't be blank"),
        length(max = 64, message = "too long user name")
    )]
    pub username: String,

    #[validate(
        length(min = 1, message = "email can't be blank"),
        length(max = 64, message = "too long email address"),
        email(message = "invalid email address")
    )]
    pub email: String,

    #[validate(
        non_control_character(message = "password can't contain non-ascii charactors"),
        length(min = 8, message = "password must be at least 8 characters long"),
        length(max = 64, message = "too long password")
    )]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, Message)]
#[rtype(result = "AppResult<UserResponse>")]
pub struct LoginUser {
    #[validate(
        email(message = "invalid email address"),
        length(min = 1, message = "email can't be blank")
    )]
    pub email: String,
    #[validate(length(min = 1, message = "password can't be blank"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, Message, Clone)]
#[rtype(result = "AppResult<UserResponse>")]
pub struct UpdateUserData {
    pub bio: Option<String>,
    pub image: Option<String>,

    #[validate(email)]
    pub email: Option<String>,
    #[validate(non_control_character, length(min = 1, max = 64))]
    pub username: Option<String>,
    #[validate(non_control_character, length(min = 8, max = 64))]
    pub password: Option<String>,
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "AppResult<UserResponse>")]
pub struct UpdateUserOuter {
    pub auth: Auth,
    pub update_user: UpdateUserData,
}

// ================================== JSON Response Objects ================================== //

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user: UserResponseInner,
}

#[derive(Debug, Serialize)]
pub struct UserResponseInner {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            user: UserResponseInner {
                token: user.generate_jwt().unwrap(),
                bio: user.bio,
                email: user.email,
                image: user.image,
                username: user.username,
            },
        }
    }
}

impl UserResponse {
    fn create_with_auth(auth: crate::utils::Auth) -> Self {
        UserResponse {
            user: UserResponseInner {
                token: auth.token,
                email: auth.user.email,
                username: auth.user.username,
                bio: auth.user.bio,
                image: auth.user.image,
            },
        }
    }
}

// ================================== Handlers ================================== //

#[actix_web::post("")]
async fn registration(
    state: web::Data<AppState>,
    form: Json<In<RegistrationUser>>,
) -> AppResult<HttpResponse> {
    let register_user = form.into_inner().user;
    register_user.validate()?;

    Ok(state.db.send(register_user).await.map(|res| match res {
        Err(e) => e.error_response(),
        Ok(res) => HttpResponse::Ok().json(res),
    })?)
}

#[actix_web::get("/login")]
pub async fn login(
    state: web::Data<AppState>,
    form: Json<In<LoginUser>>,
) -> AppResult<HttpResponse> {
    let login_user = form.into_inner().user;
    login_user.validate()?;

    Ok(state.db.send(login_user).await.map(|res| match res {
        Err(e) => e.error_response(),
        Ok(res) => HttpResponse::Ok().json(res),
    })?)
}

#[actix_web::get("")]
pub async fn get_current_user(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    authenticate(&state, &req)
        .and_then(|auth| {
            futures::future::ok(HttpResponse::Ok().json(UserResponse::create_with_auth(auth)))
        })
        .await
}

#[actix_web::put("")]
pub async fn update_user(
    req: HttpRequest,
    state: web::Data<AppState>,
    form: Json<In<UpdateUserData>>,
) -> AppResult<HttpResponse> {
    use futures::future::*;

    let update_user = form.into_inner().user;
    update_user.validate()?;

    authenticate(&state, &req)
        .and_then(|auth| async {
            Ok(state
                .db
                .send(UpdateUserOuter { auth, update_user })
                .await
                .map(|res| match res {
                    Ok(res) => HttpResponse::Ok().json(res),
                    Err(e) => e.error_response(),
                })?)
        })
        .await
}
