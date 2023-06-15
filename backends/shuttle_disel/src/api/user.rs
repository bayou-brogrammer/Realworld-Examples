use crate::error::AppResult;
use crate::utils::jwt::GenerateJwt;
use crate::{models::user::User, AppState};
use actix::Message;
use actix_web::web::{self, Json};
use actix_web::{HttpResponse, ResponseError};
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

// ================================== Handlers ================================== //

#[actix_web::post("")]
async fn registration(
    state: web::Data<AppState>,
    form: Json<In<RegistrationUser>>,
) -> AppResult<HttpResponse> {
    let register_user = form.into_inner().user;
    register_user.validate()?;

    let r = state.db.send(register_user).await?;

    println!("{:?}", r);

    match r {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(e) => Ok(e.error_response()),
    }
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
