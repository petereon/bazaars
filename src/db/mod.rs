pub mod schema;

use std::sync::Arc;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

#[derive(Clone)]
pub struct DbManager {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl DbManager {
    pub fn new(connection_string: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(connection_string);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        DbManager {
            pool: Arc::new(pool),
        }
    }

    pub fn get_write_pool(&self) -> Arc<Pool<ConnectionManager<PgConnection>>> {
        self.pool.clone()
    }

    pub fn get_read_pool(&self) -> Arc<Pool<ConnectionManager<PgConnection>>> {
        self.pool.clone()
    }
}
