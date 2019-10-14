//! User management

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use argonautica::Hasher;
use argonautica::input::SecretKey;
use diesel::prelude::*;
use lazy_static::lazy_static;
use log::{debug};
use rand::Rng;
use rand::distributions::Standard;
use serde::Serialize;

use crate::do_lock;
use crate::api::UserResource;
use crate::db::DatabaseContext;
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


#[derive(Serialize)]
pub struct User<'a, M> {
    #[serde(skip)]
    manager: &'a M,
    #[serde(flatten)]
    row: UserRow,
}

impl<'a, M> User<'a, M>
where M: UserManager
{
    pub fn delete(self) -> Result<()> {

        let conn = self.manager.conn()?;
        diesel::delete(&self.row)
            .execute(&conn)?;

        Ok(())
    }

    pub fn id(&self) -> i32 {
        self.row.id
    }

    pub fn update(&mut self, settings: UserResource) -> Result<()> {

        let mut do_save = false;

        if let Some(password) = settings.password {
            let hash = hash_password(password)?;
            self.row.pwhash = hash;
            do_save = true;
        }

        if let Some(username) = settings.username {
            if self.row.username != username {
                self.row.username = username;
                do_save = true;
            }
        }

        if do_save {
            let conn = self.manager.conn()?;
            diesel::update(&self.row)
                .set(&self.row)
                .execute(&conn)?;
        }

        Ok(())
    }
}

impl<'a, M> Into<UserResource> for User<'a, M> {
    fn into(self) -> UserResource {
        UserResource {
            id: Some(self.row.id),
            password: None,
            username: Some(self.row.username),
        }
    }
}


#[derive(Insertable)]
#[table_name = "users"]
struct NewUser {
    username: String,
    pwhash: String,
}


pub trait UserManager: DatabaseContext + Sized {

    fn create_user(
        &self,
        username: String,
        password: String,
    ) -> Result<User<Self>>
    {
        debug!("adding user {} to database", &username);
        let conn = self.conn()?;
        let pwhash = hash_password(password)?;
        let new_user = NewUser {
            username,
            pwhash,
        };
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&conn)?;

        // Get the row we just inserted
        let user_row = users::table.filter(users::username.eq(new_user.username))
            .get_result(&conn)?;
        let user = User {
            row: user_row,
            manager: self,
        };

        Ok(user)
    }

    fn get_user(&self, id: i32) -> Result<User<Self>> {

        let conn = self.conn()?;
        let user_row = users::table.find(id)
            .get_result(&conn)?;

        Ok(User {
            row: user_row,
            manager: self,
        })
    }

    fn get_users(&self) -> Result<Vec<User<Self>>> {

        let conn = self.conn()?;
        let users = users::table.load(&conn)?
            .into_iter()
            .map(|u| User {
                row: u,
                manager: self,
            })
            .collect();

        Ok(users)
    }
}

impl<T: DatabaseContext + Sized> UserManager for T {}
