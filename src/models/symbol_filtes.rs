use chrono::Utc;
use lazy_static::lazy_static;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::time::Duration;
use tokio::{sync::broadcast::Receiver, time};

use super::bean::symbol::Symbol;

lazy_static! {
    pub static ref FILTER_MAP: Arc<RwLock<HashMap<Option<String>, Filter>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

//维护交易过滤器
pub async fn maintain_symbol_filters(mut rx: Receiver<()>, pool: &'static MySqlPool) {
    let mut interval = time::interval(Duration::from_secs(60 * 10));
    loop {
        tokio::select! {
                _ = interval.tick() => {
                    let _ = update_symbol_filters(pool).await;
                }
                _ = Box::pin(rx.recv()) => {
                    break;
                }
        }
    }
}

async fn update_symbol_filters(pool: &MySqlPool) -> Result<(), Box<dyn Error>> {
    let start_time = Utc::now();
    let all_symbols = Symbol::query_all_symbols(pool).await?;
    for symbol in &all_symbols {
        let mut filter = Filter::default();
        if let Some(filters) = &symbol.filters {
            let filters: Vec<serde_json::Value> = serde_json::from_str(filters.as_str())?;
            for item in filters {
                match item["filterType"].as_str().unwrap() {
                    "PRICE_FILTER" => {
                        let price_filter: PriceFilter = serde_json::from_value(item)?;
                        filter.price_filter = Some(price_filter);
                    }
                    "LOT_SIZE" => {
                        let lot_size: LotSize = serde_json::from_value(item)?;
                        filter.lot_size = Some(lot_size);
                    }
                    "NOTIONAL" => {
                        let notional: Notional = serde_json::from_value(item)?;
                        filter.notional = Some(notional);
                    }
                    _ => {}
                }
            }
        } else {
            continue;
        }
        if let Some(filter1) = FILTER_MAP.read().unwrap().get(&symbol.symbol) {
            if filter1 != &filter {
                FILTER_MAP
                    .write()
                    .unwrap()
                    .insert(symbol.symbol.clone(), filter);
            }
        }
    }
    info!(
        "初始更新交易对过滤器数据成功，执行耗时{:?}",
        (Utc::now() - start_time).num_milliseconds()
    );
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PriceFilter {
    filter_type: String,
    min_price: String,
    max_price: String,
    tick_size: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LotSize {
    filter_type: String,
    min_qty: String,
    max_qty: String,
    step_size: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Notional {
    filter_type: String,
    avg_price_mins: String,
    min_notional: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Filter {
    notional: Option<Notional>,
    price_filter: Option<PriceFilter>,
    lot_size: Option<LotSize>,
}
