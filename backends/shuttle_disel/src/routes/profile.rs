use actix_web::{dev::HttpServiceFactory, web};

use crate::api;

pub fn profile_routes() -> impl HttpServiceFactory {
    web::scope("/profiles/{username}")
        .service(api::profile::get_profile)
        .service(api::profile::follow_profile)
        .service(api::profile::unfollow_profile)
}
