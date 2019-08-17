//! Camera management

use std::io::Write;
use diesel::backend::{Backend};
use diesel::deserialize::{self, FromSql};
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Integer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use crate::schema::cameras;


/// Video stream orientation
#[derive(Clone, Copy, Debug)]
#[derive(Deserialize, Serialize)]
#[derive(AsExpression, FromSqlRow)]
#[serde(rename_all = "camelCase")]
#[sql_type = "Integer"]
pub enum Orientation
{
    Landscape,
    Portrait,
    InvertedLandscape,
    InvertedPortrait,
}

impl<B> FromSql<Integer, B> for Orientation
where
    B: Backend,
    i32: FromSql<Integer, B>,
{
    fn from_sql(bytes: Option<&B::RawValue>) -> deserialize::Result<Self>
    {
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
    fn to_sql<W: Write>(&self, out: &mut Output<W, B>) -> serialize::Result
    {
        let val = match *self {
            Orientation::Landscape => 0,
            Orientation::Portrait => 1,
            Orientation::InvertedLandscape => 2,
            Orientation::InvertedPortrait => 3,
        };

        val.to_sql(out)
    }
}


/// Information needed to create a new camera
#[derive(Insertable)]
#[table_name = "cameras"]
pub struct NewCamera {
    pub hostname: String,
    pub device_key: String,
    pub friendly_name: String,
}


/// Represents a streaming camera controlled by this server
#[derive(AsChangeset, Identifiable, Insertable, Queryable)]
#[table_name = "cameras"]
pub struct Camera
{
    pub id: i32,
    pub friendly_name: String,
    pub hostname: String,
    pub device_key: String,
    pub enabled: bool,
    pub orientation: Orientation,
}

impl Camera
{
    /// Creates a new camera in the database
    pub fn create(camera: NewCamera, db: &SqliteConnection) -> QueryResult<Self>
    {
        use crate::schema::cameras::dsl::*;

        debug!("adding new camera to database");
        diesel::insert_into(cameras)
            .values(camera)
            .execute(db)?;

        // Yuck. But this is the only option when using SQLite.
        // https://github.com/diesel-rs/diesel/issues/376
        // TODO: this technically isn't thread-safe
        trace!("retrieving new camera from database");
        cameras.order(id.desc())
            .first(db)
    }

    /// Deletes this camera from the database
    pub fn delete(self, db: &SqliteConnection) -> QueryResult<()>
    {
        debug!("deleting camera {}", self.id);
        diesel::delete(&self)
            .execute(db)?;

        Ok(())
    }

    /// Gets all cameras from the database
    pub fn get_all(db: &SqliteConnection) -> QueryResult<Vec<Self>>
    {
        use crate::schema::cameras::dsl::*;

        trace!("retrieving all cameras from database");
        cameras.load(db)
    }

    /// Gets the specified camera from the database
    pub fn get(cam_id: i32, db: &SqliteConnection) -> QueryResult<Self>
    {
        use crate::schema::cameras::dsl::*;

        trace!("retrieving camera {} from database", cam_id);
        cameras.find(cam_id)
            .get_result(db)
    }

    /// Saves any changes to this camera to the database
    pub fn save(&self, db: &SqliteConnection) -> QueryResult<()>
    {
        debug!("saving camera {} to database", self.id);
        diesel::update(self)
            .set(self)
            .execute(db)?;

        Ok(())
    }
}
