mod api;
mod db;
mod error;
mod models;
mod routes;
mod schema;
mod utils;

use std::sync::mpsc;

use actix::{Addr, SyncArbiter, System};
use actix_web::{
    get,
    middleware::Logger,
    web::{self, ServiceConfig},
};
use db::{Conn, DbExecutor, PgPool};
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use error::AppResult;
use shuttle_actix_web::ShuttleActixWeb;
#[derive(Clone)]
pub struct AppState {
    // pub pool: PgPool,
    pub db: Addr<DbExecutor>,
}

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello World!"
}

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_shared_db::Postgres] _pool: sqlx::PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenv().ok();

    for (key, value) in std::env::vars() {
        println!("{}: {}", key, value);
    }

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
                .wrap(Logger::default())
                .service(routes::user::user_routes())
                .service(routes::user::users_routes())
                .service(routes::profile::profile_routes())
                .app_data(state),
        );
    };

    Ok(config.into())
}

pub fn new_pool<S: Into<String>>(database_url: S) -> AppResult<PgPool> {
    let manager = ConnectionManager::<Conn>::new(database_url.into());
    let pool = r2d2::Pool::builder().build(manager)?;
    Ok(pool)
}
