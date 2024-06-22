use std::thread;

use binance_spot_connector_rust::{
    market::{book_ticker, klines::KlineInterval},
    market_stream::book_ticker::BookTickerStream,
    tokio_tungstenite::BinanceWebSocketClient,
};

use chrono::NaiveDateTime;
use env_logger::Builder;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Receiver;

const BINANCE_WSS_BASE_URL: &str = "wss://testnet.binance.vision/ws";

pub async fn listen_and_maintain_book_tickers(symbol_names: Vec<String>, mut rx: Receiver<()>) {
    Builder::from_default_env()
        .filter(None, log::LevelFilter::Debug)
        .init();
    // Establish connection
    let (mut conn, _) = BinanceWebSocketClient::connect_async(BINANCE_WSS_BASE_URL)
        .await
        .expect("Failed to connect");
    // Read messages
    let mut vec_book_tickers = vec![];
    for symbol_name in &symbol_names {
        vec_book_tickers.push(BookTickerStream::from_symbol(symbol_name).into());
        // conn.subscribe(vec![&BookTickerStream::from_symbol(&symbol_name).into()])
        //     .await;
    }
    conn.subscribe(vec_book_tickers.iter()).await;
        loop {
            tokio::select! {
                Some(message) =  conn.as_mut().next() => {
                        match message {
                            Ok(message) => {
                                let data = message.into_data();
                                let string_data = String::from_utf8(data).expect("Found invalid UTF-8 chars");
                                log::info!("{}", &string_data);
                                match serde_json::from_str::<BookTicker>(&string_data){
                                    Ok(book_ticker) => {
                                        log::info!("{:?}", book_ticker);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to parse book ticker: {}", e);
                                    }
                                }

                            }
                            Err(_) => {
                                conn.close().await.expect("Failed to disconnect");
                                break
                            },

                        }
                }
                _ = Box::pin(rx.recv()) => {
                    conn.close().await.expect("Failed to disconnect");
                    break;
                }
            }
        }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookTicker {
    #[serde(rename = "u")]
    update: u64, // order book updateId
    #[serde(rename = "s")]
    symbol: String, // 交易对
    #[serde(rename = "b")]
    buy_price: String, // 买单最优挂单价格
    #[serde(rename = "B")]
    buy_quantity: String, // 买单最优挂单数量
    #[serde(rename = "a")]
    sell_price: String, // 卖单最优挂单价格
    #[serde(rename = "A")]
    sell_quantity: String, // 卖单最优挂单数量
}

#[cfg(test)]
mod tests {
    use std::thread;

    use tokio::time;

    use super::*;

    #[tokio::test]
    async fn test_listen_and_maintain_book_tickers() {
        let symbol_names = vec!["BTCUSDT".to_string(), "BNBUSDT".to_string()];
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        let h = tokio::spawn(listen_and_maintain_book_tickers(symbol_names, rx));
    
        thread::spawn(move||{
            thread::sleep(std::time::Duration::from_secs(20));
            let _ = tx.send(());
        });
        let _ = tokio::try_join!(h);
        
    }
}
