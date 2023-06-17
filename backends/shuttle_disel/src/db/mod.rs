pub mod auth;
pub use auth::*;

pub mod articles;
pub use articles::*;

pub mod profile;
pub use profile::*;

mod user;
pub use user::*;

use actix::prelude::{Actor, SyncContext};
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};

pub type Conn = PgConnection;
pub type PgPool = Pool<ConnectionManager<Conn>>;
pub type PooledConn = PooledConnection<ConnectionManager<Conn>>;

pub struct DbExecutor(pub PgPool);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
