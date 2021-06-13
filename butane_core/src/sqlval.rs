use crate::custom::{SqlValCustom, SqlValRefCustom};
use crate::{DataObject, Error::CannotConvertSqlVal, Result, SqlType};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;

#[cfg(feature = "pg")]
use crate::custom::SqlTypeCustom;

#[cfg(feature = "datetime")]
use chrono::naive::NaiveDateTime;

#[derive(Clone, Debug)]
pub enum SqlValRef<'a> {
    Null,
    Bool(bool),
    Int(i32),
    BigInt(i64),
    Real(f64),
    Text(&'a str),
    Blob(&'a [u8]),
    #[cfg(feature = "datetime")]
    Timestamp(NaiveDateTime), // NaiveDateTime is Copy
    Custom(SqlValRefCustom<'a>),
}
impl SqlValRef<'_> {
    // if this is Null
    pub fn sqltype(&self) -> Option<SqlType> {
        match self {
            SqlValRef::Null => None,
            SqlValRef::Bool(_) => Some(SqlType::Bool),
            SqlValRef::Int(_) => Some(SqlType::Int),
            SqlValRef::BigInt(_) => Some(SqlType::BigInt),
            SqlValRef::Real(_) => Some(SqlType::Real),
            SqlValRef::Text(_) => Some(SqlType::Text),
            #[cfg(feature = "datetime")]
            SqlValRef::Timestamp(_) => Some(SqlType::Timestamp),
            SqlValRef::Blob(_) => Some(SqlType::Blob),
            #[cfg(feature = "pg")]
            SqlValRef::Custom(c) => match c {
                SqlValRefCustom::PgToSql { ty, .. } => {
                    Some(SqlType::Custom(SqlTypeCustom::Pg(ty.clone())))
                }
                SqlValRefCustom::PgBytes { ty, .. } => {
                    Some(SqlType::Custom(SqlTypeCustom::Pg(ty.clone())))
                }
            },
            #[cfg(not(feature = "pg"))]
            SqlValRef::Custom(_) => None,
        }
    }
}

/// A database value.
///
/// For conversion between `SqlVal` and other types, see [`FromSql`], [`IntoSql`], and [`ToSql`].
///
/// [`FromSql`]: crate::FromSql
/// [`IntoSql`]: crate::IntoSql
/// [`ToSql`]: crate::ToSql
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SqlVal {
    Null,
    Bool(bool),
    Int(i32),
    BigInt(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
    #[cfg(feature = "datetime")]
    Timestamp(NaiveDateTime),
    Custom(Box<SqlValCustom>),
}
impl SqlVal {
    pub fn as_ref(&self) -> SqlValRef<'_> {
        SqlValRef::from(self)
    }

    pub fn bool(&self) -> Result<bool> {
        match self {
            SqlVal::Bool(val) => Ok(*val),
            _ => Err(CannotConvertSqlVal(SqlType::Bool, self.clone())),
        }
    }
    pub fn integer(&self) -> Result<i32> {
        match self {
            SqlVal::Int(val) => Ok(*val),
            _ => Err(CannotConvertSqlVal(SqlType::Int, self.clone())),
        }
    }
    pub fn bigint(&self) -> Result<i64> {
        match self {
            SqlVal::Int(val) => Ok(*val as i64),
            SqlVal::BigInt(val) => Ok(*val),
            _ => Err(CannotConvertSqlVal(SqlType::BigInt, self.clone())),
        }
    }
    pub fn real(&self) -> Result<f64> {
        match self {
            SqlVal::Real(val) => Ok(*val),
            _ => Err(CannotConvertSqlVal(SqlType::Real, self.clone())),
        }
    }
    pub fn text(&self) -> Result<&str> {
        match self {
            SqlVal::Text(val) => Ok(val),
            _ => Err(CannotConvertSqlVal(SqlType::Text, self.clone())),
        }
    }
    pub fn owned_text(self) -> Result<String> {
        match self {
            SqlVal::Text(val) => Ok(val),
            _ => Err(CannotConvertSqlVal(SqlType::Text, self.clone())),
        }
    }
    pub fn blob(&self) -> Result<&[u8]> {
        match self {
            SqlVal::Blob(val) => Ok(val),
            _ => Err(CannotConvertSqlVal(SqlType::Blob, self.clone())),
        }
    }
    pub fn owned_blob(self) -> Result<Vec<u8>> {
        match self {
            SqlVal::Blob(val) => Ok(val),
            _ => Err(CannotConvertSqlVal(SqlType::Blob, self.clone())),
        }
    }

    /// Tests if this sqlval is compatible with the given
    /// `SqlType`. There are no implicit type conversions (i.e. if
    /// this is a `SqlVal::Bool`, it is only compatible with
    /// `SqlType::Bool`, not with `SqlType::Int`, even though an int
    /// contains enough information to encode a bool.
    #[allow(unreachable_patterns)]
    pub fn is_compatible(&self, t: &SqlType, null_allowed: bool) -> bool {
        match self.sqltype() {
            None => null_allowed,
            Some(self_ty) => *t == self_ty,
        }
    }

    // Returns the SqlType most appropriate to this value or None
    // if this is Null
    pub fn sqltype(&self) -> Option<SqlType> {
        match self {
            SqlVal::Null => None,
            SqlVal::Bool(_) => Some(SqlType::Bool),
            SqlVal::Int(_) => Some(SqlType::Int),
            SqlVal::BigInt(_) => Some(SqlType::BigInt),
            SqlVal::Real(_) => Some(SqlType::Real),
            SqlVal::Text(_) => Some(SqlType::Text),
            #[cfg(feature = "datetime")]
            SqlVal::Timestamp(_) => Some(SqlType::Timestamp),
            SqlVal::Blob(_) => Some(SqlType::Blob),
            #[cfg(feature = "pg")]
            SqlVal::Custom(c) => match c.as_ref() {
                SqlValCustom::Pg { ty, .. } => Some(SqlType::Custom(SqlTypeCustom::Pg(ty.clone()))),
            },
            #[cfg(not(feature = "pg"))]
            SqlVal::Custom(_) => None,
        }
    }
}
impl fmt::Display for SqlVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use SqlVal::*;
        match &self {
            SqlVal::Null => f.write_str("NULL"),
            SqlVal::Bool(val) => val.fmt(f),
            Int(val) => val.fmt(f),
            BigInt(val) => val.fmt(f),
            Real(val) => val.fmt(f),
            Text(val) => val.fmt(f),
            Blob(val) => f.write_str(&hex::encode(val)),
            #[cfg(feature = "datetime")]
            Timestamp(val) => val.format("%+").fmt(f),
            Custom(val) => val.fmt(f),
        }
    }
}

/// Used to convert another type to a `SqlVal` or `SqlValRef`.
///
/// Unlike [`IntoSql`][crate::IntoSql], the value is not consumed.
pub trait ToSql {
    fn to_sql(&self) -> SqlVal;
    fn to_sql_ref(&self) -> SqlValRef<'_>;
}

/// Used to convert another type to a `SqlVal`.
///
/// The value is consumed. For a non-consuming trait, see
/// [`ToSql`][crate::ToSql].
pub trait IntoSql {
    fn into_sql(self) -> SqlVal;
}

impl<T> From<T> for SqlVal
where
    T: IntoSql,
{
    fn from(val: T) -> Self {
        val.into_sql()
    }
}

/// Used to convert a `SqlVal` or `SqlValRef` into another type.
///
/// The `SqlVal` is consumed.
pub trait FromSql {
    /// Used to convert a SqlValRef into another type.
    fn from_sql_ref(val: SqlValRef<'_>) -> Result<Self>
    where
        Self: Sized;

    /// Used to convert a SqlVal into another type. The default
    /// implementation calls `Self::from_sql_ref(val.as_ref())`, which
    /// may be inefficient. This method is chiefly used only for
    /// primary keys: a more efficient implementation is unlikely to
    /// provide benefits for types not used as primary keys.
    fn from_sql(val: SqlVal) -> Result<Self>
    where
        Self: Sized,
    {
        Self::from_sql_ref(val.as_ref())
    }
}

impl From<SqlValRef<'_>> for SqlVal {
    fn from(vref: SqlValRef) -> SqlVal {
        use SqlValRef::*;
        match vref {
            Null => SqlVal::Null,
            Bool(v) => SqlVal::Bool(v),
            Int(v) => SqlVal::Int(v),
            BigInt(v) => SqlVal::BigInt(v),
            Real(v) => SqlVal::Real(v),
            Text(v) => SqlVal::Text(v.to_string()),
            Blob(v) => SqlVal::Blob(v.into()),
            #[cfg(feature = "datetime")]
            Timestamp(v) => SqlVal::Timestamp(v),
            Custom(v) => SqlVal::Custom(Box::new(v.into())),
        }
    }
}

impl<'a> From<&'a SqlVal> for SqlValRef<'a> {
    fn from(val: &'a SqlVal) -> SqlValRef<'a> {
        use SqlVal::*;
        match val {
            Null => SqlValRef::Null,
            Bool(v) => SqlValRef::Bool(*v),
            Int(v) => SqlValRef::Int(*v),
            BigInt(v) => SqlValRef::BigInt(*v),
            Real(v) => SqlValRef::Real(*v),
            Text(v) => SqlValRef::Text(v.as_ref()),
            Blob(v) => SqlValRef::Blob(v.as_ref()),
            #[cfg(feature = "datetime")]
            Timestamp(v) => SqlValRef::Timestamp(*v),
            Custom(v) => SqlValRef::Custom(v.as_valref()),
        }
    }
}

/// Type suitable for being a database column.
pub trait FieldType: ToSql + IntoSql + FromSql {
    const SQLTYPE: SqlType;
    /// Reference type. Used for ergonomics with String (which has
    /// reference type str). For most, it is Self
    type RefType: ?Sized + ToSql;
}

/// Marker trait for a type suitable for being a primary key
pub trait PrimaryKeyType: FieldType + Clone + PartialEq {}

/// Trait for referencing the primary key for a given model. Used to
/// implement ForeignKey equality tests.
pub trait AsPrimaryKey<T: DataObject> {
    fn as_pk(&self) -> Cow<<T as DataObject>::PKType>;
}

impl<P, T> AsPrimaryKey<T> for P
where
    P: PrimaryKeyType,
    T: DataObject<PKType = P>,
{
    fn as_pk(&self) -> Cow<P> {
        Cow::Borrowed(&self)
    }
}

macro_rules! sql_conv_err {
    ($val:ident, $sqltype:ident) => {
        Err(crate::Error::CannotConvertSqlVal(
            SqlType::$sqltype,
            $val.into(),
        ))
    };
}

macro_rules! impl_basic_from_sql {
    ($prim:ty, $variant:ident, $sqltype:ident) => {
        impl FromSql for $prim {
            fn from_sql_ref(valref: SqlValRef) -> Result<Self> {
                if let SqlValRef::$variant(val) = valref {
                    Ok(val as $prim)
                } else {
                    sql_conv_err!(valref, $sqltype)
                }
            }
            fn from_sql(val: SqlVal) -> Result<Self> {
                if let SqlVal::$variant(val) = val {
                    Ok(val as $prim)
                } else {
                    sql_conv_err!(val, $sqltype)
                }
            }
        }
    };
}

macro_rules! impl_prim_sql {
    ($prim:ty, $variant:ident, $sqltype:ident) => {
        impl_prim_sql! {$prim, $variant, $sqltype, $prim}
    };
    ($prim:ty, $variant:ident, $sqltype:ident, $reftype: ty) => {
        impl_basic_from_sql!($prim, $variant, $sqltype);
        impl IntoSql for $prim {
            fn into_sql(self) -> SqlVal {
                SqlVal::$variant(self.into())
            }
        }
        impl ToSql for $prim {
            fn to_sql(&self) -> SqlVal {
                self.clone().into_sql()
            }
            fn to_sql_ref(&self) -> SqlValRef<'_> {
                SqlValRef::$variant(self.clone().into())
            }
        }
        impl FieldType for $prim {
            const SQLTYPE: SqlType = SqlType::$sqltype;
            type RefType = $reftype;
        }

        impl PrimaryKeyType for $prim {}
    };
}

impl_prim_sql!(bool, Bool, Bool);
impl_prim_sql!(i64, BigInt, BigInt);
impl_prim_sql!(i32, Int, Int);
impl_prim_sql!(u32, BigInt, BigInt);
// TODO need a small int type
impl_prim_sql!(u16, Int, Int);
impl_prim_sql!(i16, Int, Int);
impl_prim_sql!(u8, Int, Int);
impl_prim_sql!(i8, Int, Int);
impl_prim_sql!(f64, Real, Real);
impl_prim_sql!(f32, Real, Real);

impl FromSql for String {
    fn from_sql_ref(valref: SqlValRef) -> Result<Self> {
        if let SqlValRef::Text(val) = valref {
            Ok(val.to_string())
        } else {
            sql_conv_err!(valref, Text)
        }
    }
    fn from_sql(val: SqlVal) -> Result<Self> {
        if let SqlVal::Text(val) = val {
            Ok(val)
        } else {
            sql_conv_err!(val, Text)
        }
    }
}
impl ToSql for String {
    fn to_sql(&self) -> SqlVal {
        SqlVal::Text(self.clone())
    }
    fn to_sql_ref(&self) -> SqlValRef<'_> {
        SqlValRef::Text(self)
    }
}
impl IntoSql for String {
    fn into_sql(self) -> SqlVal {
        SqlVal::Text(self)
    }
}
impl FieldType for String {
    const SQLTYPE: SqlType = SqlType::Text;
    type RefType = str;
}
impl PrimaryKeyType for String {}

impl FromSql for Vec<u8> {
    fn from_sql_ref(valref: SqlValRef) -> Result<Self> {
        if let SqlValRef::Blob(val) = valref {
            Ok(Vec::from(val))
        } else {
            sql_conv_err!(valref, Blob)
        }
    }
    fn from_sql(val: SqlVal) -> Result<Self> {
        if let SqlVal::Blob(val) = val {
            Ok(val)
        } else {
            sql_conv_err!(val, Blob)
        }
    }
}
impl ToSql for Vec<u8> {
    fn to_sql(&self) -> SqlVal {
        SqlVal::Blob(self.clone())
    }
    fn to_sql_ref(&self) -> SqlValRef<'_> {
        SqlValRef::Blob(self)
    }
}
impl IntoSql for Vec<u8> {
    fn into_sql(self) -> SqlVal {
        SqlVal::Blob(self)
    }
}
impl FieldType for Vec<u8> {
    const SQLTYPE: SqlType = SqlType::Blob;
    type RefType = Self;
}
impl PrimaryKeyType for Vec<u8> {}

#[cfg(feature = "datetime")]
impl_basic_from_sql!(NaiveDateTime, Timestamp, Timestamp);
#[cfg(feature = "datetime")]
impl ToSql for NaiveDateTime {
    fn to_sql(&self) -> SqlVal {
        SqlVal::Timestamp(*self)
    }
    fn to_sql_ref(&self) -> SqlValRef<'_> {
        SqlValRef::Timestamp(*self)
    }
}
#[cfg(feature = "datetime")]
impl IntoSql for NaiveDateTime {
    fn into_sql(self) -> SqlVal {
        SqlVal::Timestamp(self)
    }
}
#[cfg(feature = "datetime")]
impl FieldType for NaiveDateTime {
    const SQLTYPE: SqlType = SqlType::Timestamp;
    type RefType = str;
}
#[cfg(feature = "datetime")]
impl PrimaryKeyType for NaiveDateTime {}

impl ToSql for &str {
    fn to_sql(&self) -> SqlVal {
        SqlVal::Text((*self).to_string())
    }
    fn to_sql_ref(&self) -> SqlValRef<'_> {
        SqlValRef::Text(self)
    }
}
impl ToSql for str {
    fn to_sql(&self) -> SqlVal {
        SqlVal::Text(self.to_string())
    }
    fn to_sql_ref(&self) -> SqlValRef<'_> {
        SqlValRef::Text(self)
    }
}

impl<T> ToSql for Option<T>
where
    T: ToSql,
{
    fn to_sql(&self) -> SqlVal {
        match self {
            None => SqlVal::Null,
            Some(v) => v.to_sql(),
        }
    }
    fn to_sql_ref(&self) -> SqlValRef<'_> {
        match self {
            None => SqlValRef::Null,
            Some(v) => v.to_sql_ref(),
        }
    }
}
impl<T> IntoSql for Option<T>
where
    T: IntoSql,
{
    fn into_sql(self) -> SqlVal {
        match self {
            None => SqlVal::Null,
            Some(v) => v.into_sql(),
        }
    }
}
impl<T> FromSql for Option<T>
where
    T: FromSql,
{
    fn from_sql_ref(valref: SqlValRef) -> Result<Self> {
        Ok(match valref {
            SqlValRef::Null => None,
            _ => Some(T::from_sql_ref(valref)?),
        })
    }
}
impl<T> FieldType for Option<T>
where
    T: FieldType,
{
    const SQLTYPE: SqlType = T::SQLTYPE;
    type RefType = Self;
}
