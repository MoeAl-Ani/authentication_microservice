use crate::entities::UserEntity;
use crate::user_dao;
use crate::user_dao::UserDao;
use sqlx::{MySql, Pool, MySqlPool};
use sqlx::pool::PoolConnection;

pub struct UserService<'a> {
    user_dao: UserDao<'a>
}

impl <'a> UserService<'a> {
    pub fn new(conn: &'a MySqlPool) -> Self {
        UserService {
            user_dao: UserDao::new(conn)
        }
    }

    pub async fn fetch_by_email(&mut self, email: &String) -> Option<UserEntity> {
        self.user_dao.find_by_email(email).await
    }

    pub async fn create_one(&mut self, user_entity: UserEntity) -> Option<UserEntity> {
        self.user_dao.insert_one(user_entity).await
    }
}