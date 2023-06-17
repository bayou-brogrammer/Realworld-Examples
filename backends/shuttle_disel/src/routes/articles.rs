use actix_web::{dev::HttpServiceFactory, web};

use crate::api;

pub fn article_routes() -> impl HttpServiceFactory {
    web::scope("/articles")
        .service(api::articles::get_articles)
        .service(api::articles::get_article)
        .service(api::articles::create_article)
        .service(api::articles::update_article)
        .service(api::articles::delete_article)
        .service(api::articles::favorite_article)
        .service(api::articles::unfavorite_article)
}

pub fn article_feed_routes() -> impl HttpServiceFactory {
    web::scope("/articles/feed").service(api::articles::get_feed_articles)
}
