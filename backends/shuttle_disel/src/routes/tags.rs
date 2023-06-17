use actix_web::{dev::HttpServiceFactory, web};

use crate::api;

pub fn tags_routes() -> impl HttpServiceFactory {
    web::scope("/tags").service(api::tags::get_tags)
}
