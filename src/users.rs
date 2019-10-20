//! User management

// Actix handlers have lots of needless pass-by-value (Data, Json, and Path structs)
#![allow(clippy::needless_pass_by_value)]

use std::sync::Mutex;

use actix_web::web::{self, Data, Json, ServiceConfig};
use argonautica::Hasher;
use argonautica::input::SecretKey;
use diesel::prelude::*;
use lazy_static::lazy_static;
use log::{debug, info, trace};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::db::{ConnectionPool, PooledConnection};
use crate::db::schema::users;
use crate::do_lock;
use crate::error::Result;
use crate::settings;


lazy_static! {
    static ref ARGON2_KEY: Mutex<Option<SecretKey<'static>>> = Mutex::new(None);
}


const ARGON2_KEY_SETTING: &str = "argon2SecretKey";


fn get_secret_key(conn: &PooledConnection) -> Result<SecretKey<'static>> {

    let mut key = do_lock!(ARGON2_KEY);

    if let Some(key) = key.as_ref() {
        trace!("retrieving cached secret key");
        return Ok(key.to_owned());
    }

    debug!("loading secret key from database");
    if let Some(key_b64) = settings::get::<String>(ARGON2_KEY_SETTING, conn)? {
        trace!("loaded secret key from database");
        let new_key = SecretKey::from_base64_encoded(key_b64)?;
        key.replace(new_key.to_owned());
        return Ok(new_key);
    }

    debug!("secret key not found in database, generating a new one");
    let raw_key: [u8; 32] = rand::thread_rng().gen();
    let new_key = SecretKey::from(&raw_key as &[_]);
    settings::set(ARGON2_KEY_SETTING, &new_key.to_base64_encoded(), conn)?;
    key.replace(new_key.to_owned());

    Ok(new_key.to_owned())
}


fn hash_password(password: &str, conn: &PooledConnection) -> Result<String> {

    // Don't really understand why, but declaring hasher with a separate statement forces the 'a in
    // Hasher<'a> to be compatible with the lifetime of password. If we don't do this, the compiler
    // tries to infer hasher as Hasher<'static>, probably because we're passing a SecretKey<'static>
    // to with_secret_key. This would lead to a lifetime mismatch when calling with_password,
    // because password is not 'static.
    let mut hasher = Hasher::new();

    let hash = hasher.with_secret_key(get_secret_key(conn)?)
        .with_password(password)
        .hash()?;

    Ok(hash)
}


/// Representation of a user account
#[derive(Serialize)]
#[derive(AsChangeset, Identifiable, Queryable)]
#[table_name = "users"]
struct User {
    id: i32,
    username: String,
    #[serde(skip_serializing)]
    pwhash: String,
}


/// User representation required by PUT requests
#[derive(Deserialize)]
struct PutUserBody {
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
    body: Json<PutUserBody>,
) -> Result<Json<User>>
{
    let body = body.into_inner();

    debug!("adding new user to database");
    let conn = pool.get()?;
    let new_user = NewUser {
        username: body.username,
        pwhash: hash_password(&body.password, &conn)?,
    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&conn)?;

    // Get the row we just inserted
    let user: User = users::table.filter(users::username.eq(new_user.username))
        .get_result(&conn)?;

    info!("created new user {}", user.id);

    Ok(Json(user))
}


/// Retrieves information about the specified user
fn get_user(
    pool: Data<ConnectionPool>,
    path: web::Path<(i32,)>,
) -> Result<Json<User>>
{
    let id = path.0;

    debug!("retrieving user {} from database", id);
    let conn = pool.get()?;
    let user = users::table.find(id)
        .get_result(&conn)?;

    Ok(Json(user))
}


/// Retrieves information about all users
fn get_users(
    pool: Data<ConnectionPool>,
) -> Result<Json<Vec<User>>>
{
    debug!("retrieving all users from database");
    let conn = pool.get()?;
    let users = users::table.load(&conn)?;

    Ok(Json(users))
}


/// User representation required by PATCH requests
#[derive(Deserialize)]
struct PatchUserBody {
    password: Option<String>,
    username: Option<String>,
}


/// Updates information about the specified user
fn patch_user(
    pool: Data<ConnectionPool>,
    path: web::Path<(i32,)>,
    body: Json<PatchUserBody>,
) -> Result<Json<User>>
{
    let id = path.0;
    let body = body.into_inner();

    debug!("retrieving user {} from database", id);
    let conn = pool.get()?;
    let mut user: User = users::table.find(id)
        .get_result(&conn)?;

    let mut do_save = false;

    if let Some(password) = body.password {
        trace!("updating pwhash for user {}", id);
        user.pwhash = hash_password(&password, &conn)?;
        do_save = true;
    }

    if let Some(username) = body.username {
        if username != user.username {
            trace!("updating username for user {}", id);
            user.username = username;
            do_save = true;
        }
    }

    if do_save {
        debug!("saving changes to user {}", id);
        diesel::update(&user)
            .set(&user)
            .execute(&conn)?;
    }

    info!("successfully updated user {}", id);
    Ok(Json(user))
}


/// Deletes the specified user
fn delete_user(
    pool: Data<ConnectionPool>,
    path: web::Path<(i32,)>,
) -> Result<()>
{
    let id = path.0;

    debug!("deleting user {} from database", id);
    let conn = pool.get()?;
    diesel::delete(users::table.filter(users::id.eq(id)))
        .execute(&conn)?;

    info!("deleted user {}", id);

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

    let users: Vec<User> = users::table.load(conn)?;

    Ok(users)
}
