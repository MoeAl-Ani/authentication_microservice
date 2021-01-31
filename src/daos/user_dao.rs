
use uuid::Uuid;
use sqlx::{MySql, Error, Row, Executor, Pool, MySqlPool};
use sqlx::pool::PoolConnection;
use sqlx::mysql::{MySqlRow, MySqlDone};
use std::borrow::BorrowMut;
use crate::entities::user_entity::UserEntity;

pub struct UserDao<'a> {
    conn: &'a MySqlPool
}
impl <'a> UserDao<'a> {
    pub fn new(conn: &'a MySqlPool) -> Self {
        UserDao {
            conn
        }
    }

    pub async fn find_by_email(&mut self, email: &String) -> Option<UserEntity> {
        let row = sqlx::query("SELECT * from user where user.email like ?")
            .bind(email)
            .fetch_one(self.conn).await;

        let mut f_name = None;
        let mut l_name = None;
        match row {
            Ok(r) => {
                if let Ok(first_name) = r.try_get("first_name") {
                    f_name = Some(first_name);
                }

                if let Ok(last_name) = r.try_get("last_name") {
                    l_name = Some(last_name);
                }
                let user_entity = Some(UserEntity {
                    id: r.get_unchecked("id"),
                    email: r.get("email"),
                    first_name: f_name,
                    last_name: l_name,
                    phone_number: r.get("phone_number"),
                    language_id: r.get_unchecked("language_id"),
                    salt: r.get("salt"),
                    verifier: r.get("verifier")
                });
                user_entity
            }
            Err(err) => {
                println!("{:?}", err);
                None
            }
        }
    }

    pub async fn insert_one(&mut self, e: UserEntity) -> Option<UserEntity> {
        let done: Result<MySqlDone, Error> = sqlx::query("INSERT INTO user(first_name, last_name, email, phone_number, language_id) VALUES(?,?,?,?,?)")
            .bind(&e.first_name)
            .bind(&e.last_name)
            .bind(&e.email)
            .bind(&e.phone_number)
            .bind(e.language_id).execute(self.conn).await;
        /*let affect_rows = self.conn.affected_rows();

        match affect_rows {
            1 => {
                self.conn.close(statement);
                Some(self.find_by_email(&e.email).unwrap())
            }
            _ => {
                None
            }
        }*/
        None
    }
}

#[cfg(test)]
mod test {
}