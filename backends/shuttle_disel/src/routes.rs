use crate::api::{articles, comments, profile, tags, user};
use actix_web::web;

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    web::scope("/api")
        // User routes ↓
        .service(web::resource("users").route(web::post().to(user::registration)))
        .service(web::resource("users/login").route(web::post().to(user::login)))
        .service(
            web::resource("user")
                .route(web::get().to(user::get_current_user))
                .route(web::put().to(user::update_user)),
        )
        // Profile routes ↓
        .service(web::resource("profiles/{username}").route(web::get().to(profile::get_profile)))
        .service(
            web::resource("profiles/{username}/follow")
                .route(web::post().to(profile::follow_profile))
                .route(web::delete().to(profile::unfollow_profile)),
        )
        // Article routes ↓
        .service(
            web::resource("articles")
                .route(web::get().to(articles::get_articles))
                .route(web::post().to(articles::create_article)),
        )
        .service(web::resource("articles/feed").route(web::get().to(articles::get_feed_articles)))
        .service(
            web::resource("articles/{slug}")
                .route(web::get().to(articles::get_article))
                .route(web::put().to(articles::update_article))
                .route(web::delete().to(articles::delete_article)),
        )
        .service(
            web::resource("articles/{slug}/favorite")
                .route(web::post().to(articles::favorite_article))
                .route(web::delete().to(articles::unfavorite_article)),
        )
        .service(
            web::resource("articles/{slug}/comments")
                .route(web::get().to(comments::get_comments))
                .route(web::post().to(comments::add_comment)),
        )
        .service(
            web::resource("articles/{slug}/comments/{comment_id}")
                .route(web::delete().to(comments::delete_comment)),
        )
        // Tags routes ↓
        .service(web::resource("tags").route(web::get().to(tags::get_tags)))
}
