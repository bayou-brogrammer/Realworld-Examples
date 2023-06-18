use actix_web::{dev::HttpServiceFactory, web};

use crate::api;

pub fn comment_routes() -> impl HttpServiceFactory {
    web::scope("/articles/{slug}/comments")
        .service(api::comments::add_comment)
        .service(api::comments::get_comments)
}
