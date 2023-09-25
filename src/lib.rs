mod executor;
mod operation;
mod select;

pub use form_macros::Queryable;
pub use operation::{Operation, Operational};
pub use select::{SelectQuery, WhereClause, WhereFunction};

#[cfg(feature = "any")]
use sqlx::{
    any::install_default_drivers,
    any::{AnyConnectOptions, AnyPoolOptions, AnyQueryResult, AnyRow},
    AnyConnection, AnyPool,
};

#[cfg(feature = "any")]
pub struct PoolOptions;

#[cfg(feature = "any")]
impl PoolOptions {
    pub fn new() -> AnyPoolOptions {
        install_default_drivers();
        AnyPoolOptions::new()
    }
}

// Exports
// any
#[cfg(feature = "any")]
pub type Pool = AnyPool;
#[cfg(feature = "any")]
pub type Connection = AnyConnection;
#[cfg(feature = "any")]
pub type ConnectOptions = AnyConnectOptions;
#[cfg(feature = "any")]
pub type QueryResult = AnyQueryResult;
#[cfg(feature = "any")]
pub type Row = AnyRow;
#[cfg(feature = "any")]
pub type Protocol = sqlx::Any;

// mysql
#[cfg(feature = "mysql")]
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlQueryResult, MySqlRow},
    MySqlConnection, MySqlPool,
};
#[cfg(feature = "mysql")]
pub type Pool = MySqlPool;
#[cfg(feature = "mysql")]
pub type PoolOptions = MySqlPoolOptions;
#[cfg(feature = "mysql")]
pub type Connection = MySqlConnection;
#[cfg(feature = "mysql")]
pub type DsnOptions = MySqlConnectOptions;
#[cfg(feature = "mysql")]
pub type QueryResult = MySqlQueryResult;
#[cfg(feature = "mysql")]
pub type Row = MySqlRow;

#[cfg(feature = "mysql")]
pub type Protocol = sqlx::MySql;

// general
pub use sqlx::{
    query, query::QueryAs, query_as, types::Uuid, ConnectOptions, Database, Error, Executor,
    FromRow, Result, Row as RowLike,
};
