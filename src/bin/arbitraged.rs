#![allow(dead_code, unused_imports)]
use arbitrage::{init, models};
use futures_util::FutureExt;
use lazy_static::lazy_static;
use log::info;
use models::arbitragedmod;
use sqlx::{MySql, MySqlPool, Pool};

lazy_static! {
    pub static ref MYSQL_POOL: Pool<MySql> = {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { init::run().await })
    };
}

#[tokio::main]
async fn main() {
    let _ = arbitragedmod::run(&MYSQL_POOL).await;
    info!("结束");
}
