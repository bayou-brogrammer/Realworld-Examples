mod api;
mod db;
mod error;
mod models;
mod schema;
mod utils;

use actix::{Addr, SyncArbiter, System};
use actix_web::{
    get,
    middleware::Logger,
    web::{self, ServiceConfig},
};
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use shuttle_actix_web::ShuttleActixWeb;
use std::sync::mpsc;

use crate::api::{articles, comments, profile, tags, user};
use db::{Conn, DbExecutor, PgPool};
use error::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub db: Addr<DbExecutor>,
}

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello World!"
}

pub fn new_pool<S: Into<String>>(database_url: S) -> AppResult<PgPool> {
    let manager = ConnectionManager::<Conn>::new(database_url.into());
    let pool = r2d2::Pool::builder().build(manager)?;
    Ok(pool)
}

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_shared_db::Postgres] _pool: sqlx::PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenv().ok();

    let pool = match new_pool("postgres://postgres:postgres@localhost:23935/postgres") {
        Ok(pool) => pool,
        Err(e) => panic!("Error: {}", e),
    };

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let s = System::new();

        let addr = SyncArbiter::start(4, move || DbExecutor(pool.clone()));
        tx.send(addr).unwrap();

        let _ = s.run();
    });

    let db = rx.recv().unwrap();
    let state = web::Data::new(AppState { db });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world).service(
            web::scope("/api")
                .app_data(state)
                .wrap(Logger::default())
                // User routes ↓
                .service(web::resource("users").route(web::post().to(user::registration)))
                .service(web::resource("users/login").route(web::post().to(user::login)))
                .service(
                    web::resource("user")
                        .route(web::get().to(user::get_current_user))
                        .route(web::put().to(user::update_user)),
                )
                // Profile routes ↓
                .service(
                    web::resource("profiles/{username}").route(web::get().to(profile::get_profile)),
                )
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
                .service(
                    web::resource("articles/feed")
                        .route(web::get().to(articles::get_feed_articles)),
                )
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
                .service(web::resource("tags").route(web::get().to(tags::get_tags))),
        );
    };

    Ok(config.into())
}
