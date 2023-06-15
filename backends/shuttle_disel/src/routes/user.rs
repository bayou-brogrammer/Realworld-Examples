use actix_web::{dev::HttpServiceFactory, web};

use crate::api;

pub fn user_routes() -> impl HttpServiceFactory {
    web::scope("/users")
        .service(api::user::login)
        .service(api::user::registration)
}
