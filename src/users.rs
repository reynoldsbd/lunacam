//! User management

use diesel::prelude::*;
use log::{debug};

use crate::api::UserResource;
use crate::db::DatabaseContext;
use crate::db::schema::users;
use crate::error::Result;


#[derive(AsChangeset, Identifiable, Queryable)]
#[table_name = "users"]
struct UserRow {
    id: i32,
    username: String,
    password: String, // TODO: hash
    display_name: String,
}


pub struct User<'a, M> {
    manager: &'a M,
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

        if let Some(display_name) = settings.display_name {
            if self.row.display_name != display_name {
                self.row.display_name = display_name;
                do_save = true;
            }
        }

        if let Some(password) = settings.password {
            if self.row.password != password {
                self.row.password = password; // TODO: hash
                do_save = true;
            }
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
            display_name: Some(self.row.display_name),
            id: Some(self.row.id),
            password: None,
            username: Some(self.row.username),
        }
    }
}


#[derive(Insertable)]
#[table_name = "users"]
struct NewUser {
    display_name: String,
    password: String, // TODO: hash
    username: String,
}


pub trait UserManager: DatabaseContext + Sized {

    fn create_user(
        &self,
        username: String,
        password: String,
        display_name: String,
    ) -> Result<User<Self>>
    {
        debug!("adding user {} to database", &username);
        let conn = self.conn()?;
        let new_user = NewUser {
            display_name,
            password,
            username,
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
