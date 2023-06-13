mod disel_service;
mod error;
mod models;
mod routes;
mod schema;

use disel_service::PgPool;

#[shuttle_runtime::main]
async fn axum(#[disel_service::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    use self::schema::posts::dsl::*;

    // let mut conn = pool.get().await.unwrap();

    // let results = posts
    //     .filter(published.eq(true))
    //     .limit(5)
    //     .select(Post::as_select())
    //     .load(&mut conn)
    //     .await;

    // println!("results: {:?}", results);

    Ok(routes::axum_service())
}
