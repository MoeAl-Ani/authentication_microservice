use crate::entities::UserEntity;
use crate::user_dao;
use crate::user_dao::UserDao;
use mysql::Conn;

pub struct UserService<'a> {
    user_dao: UserDao<'a>
}

impl <'a> UserService<'a> {
    pub fn new(conn: &'a mut Conn) -> Self {
        UserService {
            user_dao: UserDao::new(conn)
        }
    }

    pub fn fetch_by_email(&mut self, email: &String) -> Option<UserEntity> {
        self.user_dao.find_by_email(email)
    }

    pub fn create_one(&mut self, user_entity: UserEntity) -> Option<UserEntity> {
        self.user_dao.insert_one(user_entity)
    }
}