use actix_web::{dev::HttpServiceFactory, web};

use crate::api;

pub fn users_routes() -> impl HttpServiceFactory {
    web::scope("/users")
        .service(api::user::login)
        .service(api::user::registration)
        .service(web::scope("/user").service(api::user::get_current_user))
}

pub fn user_routes() -> impl HttpServiceFactory {
    web::scope("/user")
        .service(api::user::get_current_user)
        .service(api::user::update_user)
}
