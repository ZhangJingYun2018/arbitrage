use crate::apiclient::httpapiclient;
use binance_spot_connector_rust::{ureq::Error, wallet};
use chrono::Utc;
use lazy_static::lazy_static;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::time::Duration;
use tokio::{sync::broadcast::Receiver, time};

use super::bean::symbol::Symbol;

lazy_static! {
    pub static ref MAP_ACCOUNT_ASSET_BALANCE: Arc<RwLock<HashMap<Option<String>, AssetBalance>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

// 维护账户资产余额
pub async fn maintain_account_asset_balance(mut rx: Receiver<()>) {
    let mut interval = time::interval(Duration::from_secs(60 * 10));
    loop {
        tokio::select! {
                _ = interval.tick() => {
                    let _ = update_account_asset_balance().await;
                }
                _ = Box::pin(rx.recv()) => {
                    break;
                }
        }
    }
}

async fn update_account_asset_balance() -> Result<(), Box<Error>> {
    let start_time = Utc::now();
    let request = wallet::user_asset();
    let data = httpapiclient::new_http_apiclient()
        .send(request)?
        .into_body_str()?;
    let asset_balances: Vec<AssetBalance> = serde_json::from_str(&data).unwrap();
    asset_balances.into_iter().for_each(|asset_balance| {
        let mut map = MAP_ACCOUNT_ASSET_BALANCE.write().unwrap();
        map.insert(Some(asset_balance.asset.clone()), asset_balance);
    });
    info!(
        "维护账户资产余额执行完毕，执行耗时{:?}",
        (Utc::now() - start_time).num_milliseconds()
    );
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    asset: String,
    free: String,
    locked: String,
    freeze: String,
    withdrawing: String,
    ipoable: String,
    btc_valuation: String,
}
