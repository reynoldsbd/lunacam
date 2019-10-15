//! User management

// Actix handlers have lots of needless pass-by-value (Data, Json, and Path structs)
#![allow(clippy::needless_pass_by_value)]

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use actix_web::web::{self, Data, Json, ServiceConfig};
use argonautica::Hasher;
use argonautica::input::SecretKey;
use diesel::prelude::*;
use lazy_static::lazy_static;
use rand::Rng;
use rand::distributions::Standard;
use serde::{Deserialize, Serialize};

use crate::db::{ConnectionPool, PooledConnection};
use crate::do_lock;
use crate::db::schema::users;
use crate::error::Result;


lazy_static! {
    static ref ARGON2_KEY: Mutex<Option<SecretKey<'static>>> = Mutex::new(None);
}


fn get_secret_key() -> Result<SecretKey<'static>> {

    let mut key = do_lock!(ARGON2_KEY);
    if key.is_none() {

        let keyfile = format!("{}/secret-key", env::var("STATE_DIRECTORY")?);
        let keyfile = Path::new(&keyfile);
        if keyfile.exists() {

            let key_b64 = fs::read_to_string(keyfile)?;
            let new_key = SecretKey::from_base64_encoded(key_b64)?;
            key.replace(new_key);

        } else {

            let raw_key: Vec<u8> = rand::thread_rng()
                .sample_iter(Standard)
                .take(32)
                .collect();
            let new_key = SecretKey::from(raw_key);
            fs::write(keyfile, new_key.to_base64_encoded())?;
            key.replace(new_key);
        }
    }

    Ok(key.as_ref().unwrap().to_owned())
}


fn hash_password(password: String) -> Result<String> {

    let hash = Hasher::new()
        .with_secret_key(get_secret_key()?)
        .with_password(password)
        .hash()?;

    Ok(hash)
}


#[derive(Serialize)]
#[derive(AsChangeset, Identifiable, Queryable)]
#[table_name = "users"]
struct UserRow {
    id: i32,
    username: String,
    pwhash: String,
}


/// User representation returned by API requests
///
/// TODO: making this queryable would eliminate lots of manual conversion
#[derive(Serialize)]
struct UserResource {
    id: i32,
    username: String,
}


/// User representation required by PUT requests
#[derive(Deserialize)]
struct UserPut {
    password: String,
    username: String,
}


/// Used when creating a new user record with Diesel
#[derive(Insertable)]
#[table_name = "users"]
struct NewUser {
    username: String,
    pwhash: String,
}


/// Creates a new user
fn put_user(
    pool: Data<ConnectionPool>,
    user: Json<UserPut>,
) -> Result<Json<UserResource>>
{
    let conn = pool.get()?;
    let user = user.into_inner();

    let pwhash = hash_password(user.password)?;
    let new_user = NewUser {
        username: user.username,
        pwhash,
    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&conn)?;

    // Get the row we just inserted
    let row: UserRow = users::table.filter(users::username.eq(new_user.username))
        .get_result(&conn)?;

    Ok(Json(UserResource {
        id: row.id,
        username: row.username,
    }))
}


/// Retrieves information about the specified user
fn get_user(
    pool: Data<ConnectionPool>,
    path: web::Path<(i32,)>,
) -> Result<Json<UserResource>>
{
    let conn = pool.get()?;
    let id = path.0;
    let row: UserRow = users::table.find(id)
        .get_result(&conn)?;

    Ok(Json(UserResource {
        id,
        username: row.username,
    }))
}


/// Retrieves information about all users
fn get_users(
    pool: Data<ConnectionPool>,
) -> Result<Json<Vec<UserResource>>>
{
    let conn = pool.get()?;

    let users = users::table.load(&conn)?
        .into_iter()
        .map(|u: UserRow| UserResource {
            id: u.id,
            username: u.username,
        })
        .collect();

    Ok(Json(users))
}


/// User representation required by PATCH requests
#[derive(Deserialize)]
struct UserPatch {
    password: Option<String>,
    username: Option<String>,
}


/// Updates information about the specified user
fn patch_user(
    pool: Data<ConnectionPool>,
    path: web::Path<(i32,)>,
    user: Json<UserPatch>,
) -> Result<Json<UserResource>>
{
    let conn = pool.get()?;
    let id = path.0;
    let user = user.into_inner();

    let mut row: UserRow = users::table.find(id)
        .get_result(&conn)?;

    let mut do_save = false;

    if let Some(password) = user.password {
        row.pwhash = hash_password(password)?;
        do_save = true;
    }

    if let Some(username) = user.username {
        if username != row.username {
            row.username = username;
            do_save = true;
        }
    }

    if do_save {
        diesel::update(&row)
            .set(&row)
            .execute(&conn)?;
    }

    Ok(Json(UserResource {
        id,
        username: row.username,
    }))
}


/// Deletes the specified user
fn delete_user(
    pool: Data<ConnectionPool>,
    path: web::Path<(i32,)>,
) -> Result<()>
{
    let conn = pool.get()?;

    diesel::delete(users::table.filter(users::id.eq(path.0)))
        .execute(&conn)?;

    Ok(())
}


/// Configures the */users* API resource
pub fn configure_api(service: &mut ServiceConfig) {

    service.route("/users", web::get().to(get_users));
    service.route("/users", web::put().to(put_user));
    service.route("/users/{id}", web::get().to(get_user));
    service.route("/users/{id}", web::patch().to(patch_user));
    service.route("/users/{id}", web::delete().to(delete_user));
}


/// Retrieves serializable representation of all users
pub fn all(conn: &PooledConnection) -> Result<impl Serialize> {

    let users: Vec<UserRow> = users::table.load(conn)?;

    Ok(users)
}
