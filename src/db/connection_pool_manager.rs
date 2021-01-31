use std::{fs, process};
use serde::{Serialize, Deserialize};
use serde_json;
use log::{error, LevelFilter};
use sqlx::{MySqlPool, MySql, Pool, ConnectOptions};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::time::Duration;

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
    pub async fn init() -> MySqlPool {
        let config_json = fs::read_to_string("./mysql_configuration.json").unwrap_or_else(|err| {
            error!("error reading mysql config file : {}", err);
            process::exit(1);
        });

        let config: PoolConfig = serde_json::from_str(config_json.as_str()).unwrap_or_else(|err| {
            error!("error deserializing mysql config json : {}", err);
            process::exit(1);
        });

        let mut op = MySqlConnectOptions::new()
            .username(config.username.as_str())
            .password(config.password.as_str())
            .host(config.address.as_str())
            .port(config.port)
            .database(config.database.as_str());

        op.log_slow_statements(LevelFilter::Debug, Duration::new(10,0));
        op.log_statements(LevelFilter::Off);

        MySqlPoolOptions::new()
            .max_connections(100)
            .connect_with(op).await.unwrap()

    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::task::Poll;
}