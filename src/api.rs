//! Web API types

use std::io::Write;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Integer;
use serde::{Deserialize, Serialize};


/// Video stream orientation
#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(AsExpression, FromSqlRow)]
#[sql_type = "Integer"]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Orientation {
    Landscape,
    Portrait,
    InvertedLandscape,
    InvertedPortrait,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Landscape
    }
}

impl<B> FromSql<Integer, B> for Orientation
where
    B: Backend,
    i32: FromSql<Integer, B>,
{
    fn from_sql(bytes: Option<&B::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(Self::Landscape),
            1 => Ok(Self::Portrait),
            2 => Ok(Self::InvertedLandscape),
            3 => Ok(Self::InvertedPortrait),
            other => Err(format!("Unrecognized value \"{}\"", other).into()),
        }
    }
}

impl<B> ToSql<Integer, B> for Orientation
where
    B: Backend,
    i32: ToSql<Integer, B>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, B>) -> serialize::Result {
        let val = match *self {
            Self::Landscape => 0,
            Self::Portrait => 1,
            Self::InvertedLandscape => 2,
            Self::InvertedPortrait => 3,
        };

        val.to_sql(out)
    }
}


/// Video stream settings
#[derive(Clone, Copy, Default)]
#[derive(Deserialize, Serialize)]
pub struct StreamSettings {
    pub enabled: Option<bool>,
    pub orientation: Option<Orientation>,
}


/// Streaming camera settings
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraSettings {
    pub enabled: Option<bool>,
    pub hostname: Option<String>,
    pub id: Option<i32>,
    pub device_key: Option<String>,
    pub friendly_name: Option<String>,
    pub orientation: Option<Orientation>,
}


/// API representation of a user account
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResource {
    pub display_name: Option<String>,
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub username: Option<String>,
}
