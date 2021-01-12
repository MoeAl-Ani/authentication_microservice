use crate::error_base::ErrorCode;
use crate::entities::UserEntity;
use mysql::{Pool, Value, Row, Conn};
use mysql::prelude::{TextQuery, Queryable};
use mysql::params::Params;
use mysql::params;
use mysql::prelude::*;
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
        let row: Row = self.conn.exec_first(&statement, mysql::params! {
        "email" => email
    }).unwrap().unwrap();
        self.conn.close(statement);
        println!("row = {:?}", row);
        None
    }

    pub fn insert_one(&self, user: UserEntity) -> Result<(), ErrorCode>{
        Ok(())
    }
}

#[cfg(test)]
mod test {
}