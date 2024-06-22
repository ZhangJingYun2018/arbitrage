use env_logger::{Builder, Env};
use log::info;
use serde::Deserialize;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::fs;
use std::io::Write;

#[derive(Deserialize)]
struct Config {
    database: DatabaseConfig,
    log: LogConfig,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
    min_connections: u32,
    // connect_timeout: u64,
    idle_timeout: u64,
}

#[derive(Deserialize)]
struct LogConfig {
    level: String,
}

fn init_logger(log_level: &str) {
    let env = Env::default().default_filter_or(log_level);
    Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} - {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.args()
            )
        })
        .init();
}

pub async fn run() -> Pool<MySql> {
    // 读取并解析配置文件
    let read_to_string = fs::read_to_string("config.yaml").expect("Failed to read config file");
    let config_content = read_to_string;
    let config: Config =
        serde_yaml::from_str(&config_content).expect("Failed to parse config file");

    // 初始化日志
    init_logger(&config.log.level);
    info!("Connecting to database at {}", config.database.url);

    // 创建数据库连接池
    MySqlPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        // .connect_timeout(std::time::Duration::new(config.database.connect_timeout, 0))
        .idle_timeout(std::time::Duration::new(config.database.idle_timeout, 0))
        .connect(&config.database.url)
        .await
        .expect("Failed to create pool")
}
