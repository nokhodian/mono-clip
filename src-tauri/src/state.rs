use crate::db::connection::DbConn;

pub struct AppState {
    pub db: DbConn,
}

impl AppState {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}
