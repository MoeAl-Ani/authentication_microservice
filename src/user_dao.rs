use crate::error_base::ErrorCode;
use std::borrow::{BorrowMut, Borrow};

#[derive(Clone, Debug)]
pub struct UserEntity {
    id: i32,
    first_name: &'static str,
    last_name: &'static str,
    email: &'static str
}

#[derive(Clone, Debug)]
struct Database(Option<Vec<UserEntity>>);

impl Database {
    fn new() -> Self {
        Database {
            0: None,
        }
    }

    fn init_data(&mut self) {
        self.0 = Some(Vec::new());
        let mut vec = self.0.borrow_mut();
        match vec {
            None => {}
            Some(db) => {
                db.push(UserEntity {id:1, first_name: "moe", last_name: "alani", email: "moe@gmail"},);
                db.push(UserEntity {id:1, first_name: "ahmed", last_name: "bishree", email: "ahmed@gmail"},);
            }
        }

        println!("{:?}", vec.as_ref())
    }
}
static mut DATABASE:  Database = Database(None);

pub fn find_by_email(email: &str) -> Option<UserEntity> {
    /// TODO fetch from database
    find_from_db_by_email(email)
}

pub fn insert_one(user: UserEntity) -> Result<(), ErrorCode>{
    /// TODO insert to database
    unsafe {
        match DATABASE.0.as_mut() {
            None => {}
            Some(vec) => {
                vec.push(user);
                println!("{:?}", vec);
            }
        }
    }
    Ok(())
}

fn init() {
    unsafe {
        let mut cunt = &mut DATABASE;
        cunt.init_data();
    }
}

fn find_from_db_by_email(email: &str) -> Option<UserEntity> {
    unsafe {
        match  DATABASE.0.as_ref() {
            None => {
                return None
            }
            Some(vec) => {
                for user in vec {
                    if user.email == email {
                        return Some(user.clone());
                    }
                }
            }
        };
        return None
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fetch_user() {
        init();
        let option = find_by_email("moe@gmail");
        match option {
            None => {
                panic!("no user found")
            }
            Some(user) => {
                assert_eq!(user.email, "moe@gmail")
            }
        }
    }

    #[test]
    fn test_fetch_user_not_found() {
        init();
        let option = find_by_email("xxx@gmail");
        match option {
            None => {

            }
            Some(user) => {
                panic!()
            }
        }
    }

    #[test]
    fn test_insert_user() {
        init();
        insert_one(UserEntity{id: 12, email: "cunt@gmail", first_name: "ass", last_name: "ass2"});
        let option = find_by_email("cunt@gmail");
        match option {
            None => {
                panic!("no user found")
            }
            Some(user) => {
                assert_eq!(user.email, "cunt@gmail")
            }
        }

    }
}