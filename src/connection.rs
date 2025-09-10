use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::PgConnection;
use crate::config::AppConfig;
use std::sync::OnceLock;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

static DB_POOL: OnceLock<DbPool> = OnceLock::new();

pub fn get_db_pool() -> &'static DbPool {
    DB_POOL.get_or_init(|| {
        let app_config = AppConfig::instance();
        let manager = ConnectionManager::<PgConnection>::new(&app_config.database.url);
        Pool::builder()
            .build(manager)
            .expect("Failed to create DB pool")
    })
}

pub fn get_db_connection() -> PooledConnection<ConnectionManager<PgConnection>> {
    get_db_pool().get().expect("Failed to get DB connection from pool")
}