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
        let option = self.user_dao.find_by_email(email);
        None
    }
}