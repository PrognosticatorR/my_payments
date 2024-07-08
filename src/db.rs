use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

use crate::config::Configs;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(Configs::get_app_configs().database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
