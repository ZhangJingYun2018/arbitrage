use crate::apiclient::httpapiclient;
use binance_spot_connector_rust::{ureq::Error, wallet};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::time;
lazy_static! {
    pub static ref FEE_MAP: Arc<RwLock<HashMap<String, Fee>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

// 维护手续费信息
pub async fn maintain_commission_fees(mut rx: Receiver<()>) -> Result<(), Box<Error>> {
    let mut interval = time::interval(Duration::from_secs(60 * 10));
    loop {
        tokio::select! {
                _ = interval.tick() => {
                    update_commission_fees().await?;
                }
                _ = Box::pin(rx.recv()) => {
                    break;
                }
        }
    }
    Ok(())
}

async fn update_commission_fees() -> Result<(), Box<Error>> {
    let start_time = Utc::now();
    let request = wallet::trade_fee();
    let data = httpapiclient::new_http_apiclient()
        .send(request)?
        .into_body_str()?;

    let trade_fees: Vec<TradeFee> = serde_json::from_str(&data).unwrap();
    trade_fees.iter().for_each(|trade_fee| {
        let start_time = Utc::now();
        let symbol = trade_fee.symbol.clone();
        if let Some(fee) = FEE_MAP.read().unwrap().get(&symbol) {
            if fee.maker_commission != trade_fee.maker_commission
                || fee.taker_commission != trade_fee.taker_commission
            {
                let mut fee_map = FEE_MAP.write().unwrap();
                fee_map.insert(
                    symbol,
                    Fee {
                        maker_commission: trade_fee.maker_commission.clone(),
                        taker_commission: trade_fee.taker_commission.clone(),
                    },
                );
            }
            info!(
                "花费时间{:?},symbol: {}, maker_commission: {}, taker_commission: {}",
                (Utc::now() - start_time).num_milliseconds(),
                trade_fee.symbol,
                trade_fee.maker_commission,
                trade_fee.taker_commission
            );
        }
    });
    info!(
        "维护手续费信息花费时间{:?}",
        (Utc::now() - start_time).num_milliseconds()
    );
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeFee {
    pub symbol: String,
    pub maker_commission: String,
    pub taker_commission: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fee {
    pub maker_commission: String,
    pub taker_commission: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_commission_fees() {
        let result = update_commission_fees().await;
        println!("result: {:#?}", result);
        assert!(result.is_ok());
    }
}
