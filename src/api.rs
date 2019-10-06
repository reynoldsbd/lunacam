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
            0 => Ok(Orientation::Landscape),
            1 => Ok(Orientation::Portrait),
            2 => Ok(Orientation::InvertedLandscape),
            3 => Ok(Orientation::InvertedPortrait),
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
            Orientation::Landscape => 0,
            Orientation::Portrait => 1,
            Orientation::InvertedLandscape => 2,
            Orientation::InvertedPortrait => 3,
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
