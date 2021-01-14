use crate::error_base::ErrorCode;
use crate::entities::UserEntity;
use mysql::{Pool, Value, Row, Conn, Error, TxOpts};
use mysql::prelude::{TextQuery, Queryable};
use mysql::params::Params;
use mysql::params;
use mysql::prelude::*;
use uuid::Uuid;

pub struct UserDao<'a> {
    conn: &'a mut Conn
}
impl <'a> UserDao<'a> {
    pub fn new(conn: &'a mut Conn) -> Self {
        UserDao {
            conn
        }
    }

    pub fn find_by_email(&mut self, email: &String) -> Option<UserEntity> {
        let statement = self.conn.prep("SELECT * from user where user.email like :email").unwrap();
        let mut row: Row = self.conn.exec_first(&statement, mysql::params! {
        "email" => email
    }).unwrap().unwrap();
        let user_entity = Some(UserEntity {
            id: row.take("id").unwrap(),
            email: row.take("email").unwrap(),
            first_name: row.take("first_name").unwrap_or_default(),
            last_name: row.take("last_name").unwrap_or_default(),
            phone_number: row.take("phone_number").unwrap(),
            language_id: row.take("language_id").unwrap()
        });
        self.conn.close(statement);
        println!("row = {:?}", user_entity);
        user_entity
    }

    pub fn insert_one(&mut self, e: UserEntity) -> Option<UserEntity>{
        let statement = self.conn.prep(r"INSERT INTO user(first_name, last_name, email, phone_number, language_id) VALUES(:first_name,:last_name,:email,:phone_number,:language_id)").unwrap();
        let result: Option<Row> = self.conn.exec_first(&statement,
                                          mysql::params! {
                   "first_name" => &e.first_name,
                   "last_name" => &e.last_name,
                   "email" => &e.email,
                   "phone_number" => &e.phone_number,
                   "language_id" => e.language_id
                }).unwrap();
        let affect_rows = self.conn.affected_rows();

        match affect_rows {
            1 => {
                self.conn.close(statement);
                Some(self.find_by_email(&e.email).unwrap())
            }
            _ => {
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
}