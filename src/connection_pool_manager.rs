use mysql::*;
use mysql::prelude::*;
use std::{fs, process};
use serde::{Serialize, Deserialize};
use serde_json;
use log::error;

#[derive(Debug)]
pub struct PoolInstantiate;

#[derive(Deserialize, Debug)]
struct PoolConfig {
    address: String,
    port: u16,
    database: String,
    username: String,
    password: String
}

impl PoolInstantiate {
    pub fn init() -> Pool {
        let config_json = fs::read_to_string("./mysql_configuration.json").unwrap_or_else(|err| {
            error!("error reading mysql config file : {}", err);
            process::exit(1);
        });

        let config: PoolConfig = serde_json::from_str(config_json.as_str()).unwrap_or_else(|err| {
            error!("error deserializing mysql config json : {}", err);
            process::exit(1);
        });


        let opts = OptsBuilder::new()
            .user(Some(config.username))
            .pass(Some(config.password))
            .ip_or_hostname(Some(config.address))
            .tcp_port(config.port)
            .db_name(Some(config.database))
            .stmt_cache_size(0);

        Pool::new(opts).unwrap()
    }
}

pub struct ConnectionHolder {
    pub conn: Conn
}

impl ConnectionHolder {
    pub fn new(conn: Conn) -> Self {
        ConnectionHolder {
            conn
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pool_created() {
        let pool = PoolInstantiate::init();
        let result = pool.get_conn().unwrap().unwrap();
        println!("{:?}", result);
    }
}