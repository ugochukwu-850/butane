use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::{Eq, PartialEq};
use std::default::Default;
use thiserror::Error as ThisError;

#[cfg(feature = "datetime")]
mod datetime;
pub mod db;
pub mod fkey;
pub mod many;
pub mod migrations;
pub mod query;
pub mod sqlval;

#[cfg(feature = "uuid")]
pub mod uuid;

use db::internal::{Column, ConnectionMethods, Row};

pub use query::Query;
pub use sqlval::*;

pub type Result<T> = std::result::Result<T, crate::Error>;

#[derive(Clone, Default, Debug)]
pub struct ObjectState {
    pub saved: bool,
}
impl PartialEq<ObjectState> for ObjectState {
    fn eq(&self, _other: &ObjectState) -> bool {
        // Always return true. This effectively removes ObjectState
        // from participating in equality tests between two objects
        true
    }
}
impl Eq for ObjectState {}

/// A type which may be the result of a database query.
///
/// Every result type must have a corresponding object type and the
/// columns of the result type must be a subset of the columns of the
/// object type. The purpose of a result type which is not also an
/// object type is to allow a query to retrieve a subset of an
/// object's columns.
pub trait DataResult: Sized {
    /// Corresponding object type.
    type DBO: DataObject;
    type Fields: Default;
    const COLUMNS: &'static [Column];
    fn from_row(row: Row) -> Result<Self>
    where
        Self: Sized;
    /// Create a blank query (matching all rows) for this type.
    fn query() -> Query<Self>;
}

/// An object in the database.
///
/// Rather than implementing this type manually, use the
/// `#[model]` attribute.
pub trait DataObject: DataResult<DBO = Self> {
    /// The type of the primary key field.
    type PKType: FieldType + Clone + PartialEq;
    /// The name of the primary key column.
    const PKCOL: &'static str;
    /// The name of the table.
    const TABLE: &'static str;
    /// Get the primary key
    fn pk(&self) -> &Self::PKType;
    /// Find this object in the database based on primary key.
    fn get(conn: &impl ConnectionMethods, id: impl Borrow<Self::PKType>) -> Result<Self>
    where
        Self: Sized,
    {
        <Self as DataResult>::query()
            .filter(query::BoolExpr::Eq(
                Self::PKCOL,
                query::Expr::Val(id.borrow().to_sql()),
            ))
            .limit(1)
            .load(conn)?
            .into_iter()
            .nth(0)
            .ok_or(Error::NoSuchObject.into())
    }
    /// Save the object to the database.
    fn save(&mut self, conn: &impl ConnectionMethods) -> Result<()>;
    /// Delete the object from the database.
    fn delete(&self, conn: &impl ConnectionMethods) -> Result<()>;
}

pub trait ModelTyped {
    type Model: DataObject;
}

/// Propane errors.
#[derive(Debug, ThisError)]
pub enum Error {
    #[error("No such object exists")]
    NoSuchObject,
    #[error("Index out of bounds {0}")]
    BoundsError(String),
    #[error("Type mismatch converting SqlVal. Expected {0}, found value {1:?}")]
    CannotConvertSqlVal(SqlType, SqlVal),
    #[error("Mismatch between sql types and rust types while loading data for column {0}.")]
    SqlResultTypeMismatch(String),
    #[error("SqlType not known for {ty}")]
    UnknownSqlType { ty: String },
    #[error("Value has not been loaded from the database")]
    ValueNotLoaded,
    #[error("Not initialized")]
    NotInitialized,
    #[error("Already initialized")]
    AlreadyInitialized,
    #[error("Migration error {0}")]
    MigrationError(String),
    #[error("Unknown backend {0}")]
    UnknownBackend(String),
    #[error("Range error")]
    OutOfRange,
    #[error("Internal logic error")]
    Internal,
    #[error("Cannot resolve type {0}")]
    CannotResolveType(String),
    #[error("(De)serialization error {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("IO error {0}")]
    IO(#[from] std::io::Error),
    #[error("Sqlite error {0}")]
    SQLite(#[from] rusqlite::Error),
    #[error("Sqlite error {0}")]
    SQLiteFromSQL(rusqlite::types::FromSqlError),
    #[cfg(feature = "datetime")]
    #[error("Chrono error {0}")]
    Chrono(#[from] chrono::ParseError),
}

impl From<rusqlite::types::FromSqlError> for Error {
    fn from(e: rusqlite::types::FromSqlError) -> Self {
        use rusqlite::types::FromSqlError;
        match &e {
            FromSqlError::InvalidType => Error::SqlResultTypeMismatch("unknown".to_string()),
            FromSqlError::OutOfRange(_) => Error::OutOfRange,
            FromSqlError::Other(_) => Error::SQLiteFromSQL(e),
        }
    }
}

/// Enumeration of the types a database value may take.
///
/// See also [`SqlVal`][crate::SqlVal].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SqlType {
    Bool,
    /// 4 bytes
    Int,
    /// 8 bytes
    BigInt,
    /// 8 byte float
    Real,
    Text,
    // TODO properly support and test timestamp
    #[cfg(feature = "datetime")]
    Timestamp,
    // TODO properly test blob
    Blob,
}
impl std::fmt::Display for SqlType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use SqlType::*;
        match &self {
            Bool => "bool",
            Int => "int",
            BigInt => "big int",
            Real => "float",
            Text => "string",
            #[cfg(feature = "datetime")]
            Timestamp => "timestamp",
            Blob => "blog",
        }
        .fmt(f)
    }
}
