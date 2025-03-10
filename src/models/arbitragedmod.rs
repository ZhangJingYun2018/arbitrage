use super::bean::arbitragecombine::ArbitrageCombine;
use super::bookticker::BookTicker;
use log::{error, info};
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::broadcast::{self, Receiver};
use tokio::time::{self, interval, sleep};

type CombineMap = Arc<Mutex<HashMap<String, Vec<ArbitrageCombine>>>>;

pub async fn run(pool: &'static MySqlPool) -> Result<(), sqlx::Error> {
    // 从MySQL查询所有套利组合
    let all_combines = ArbitrageCombine::query_all_arbitrage_combines(pool).await?;
    let map_symbol_combines: CombineMap = Arc::new(Mutex::new(HashMap::new()));
    for combine in all_combines {
        if combine.asset0.as_ref().unwrap() != "USDT"
            || combine.asset1.as_ref().unwrap() == "BNB"
            || combine.asset2.as_ref().unwrap() == "BNB"
            || combine.symbol0.as_ref().unwrap() == "BUSDUSDT"
            || combine.symbol0.as_ref().unwrap() == "BUSDTRY"
            || combine.symbol0.as_ref().unwrap() == "USDTTRY"
        {
            continue;
        }

        let symbols = vec![
            combine.symbol0.clone(),
            combine.symbol1.clone(),
            combine.symbol2.clone(),
        ];

        for symbol in symbols {
            let mut map = map_symbol_combines.lock().unwrap();
            map.entry(symbol.unwrap())
                .or_insert_with(|| Vec::with_capacity(10))
                .push(combine.clone());
        }
    }

    let mut symbol_names = Vec::with_capacity(2000);
    {
        let map = map_symbol_combines.lock().unwrap();
        for symbol in map.keys() {
            symbol_names.push(symbol.clone());
        }
    }
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            watch_free().await;
        }
    });

    let (tx, _rx) = broadcast::channel(1);
    let rx1 = tx.subscribe();
    let rx2 = tx.subscribe();
    let rx3 = tx.subscribe();
    let rx4 = tx.subscribe();

    use super::accout::maintain_account_asset_balance;
    use super::bookticker::listen_and_maintain_book_tickers;
    use super::fee::maintain_commission_fees;
    use super::symbol_filtes::maintain_symbol_filters;

    let handle_1 = thread::spawn(move || {
        Runtime::new()
            .unwrap()
            .block_on(maintain_commission_fees(rx1))
    });
    let handle_2 = thread::spawn(move || {
        Runtime::new()
            .unwrap()
            .block_on(maintain_symbol_filters(rx2, pool))
    });
    let handle_3 = thread::spawn(move || {
        Runtime::new()
            .unwrap()
            .block_on(maintain_account_asset_balance(rx3))
    });

    let handle_4 = thread::spawn(move || {
        Runtime::new()
            .unwrap()
            .block_on(listen_and_maintain_book_tickers(symbol_names, rx4))
    });
    let _r  = handle_1.join().unwrap();
    handle_2.join().unwrap();
    handle_3.join().unwrap();
    handle_4.join().unwrap();
    let _ = tx.send(());

    Ok(())
}

async fn watch_free() {}

// async fn trigger_arbitrage_analysis_and_order(
//     trigger_symbol: String,
//     trigger_book_ticker: BookTicker,
//     map_symbol_combines: CombineMap,
// ) {
//     let start_at = Instant::now();
//     let combines = {
//         let map = map_symbol_combines.lock().unwrap();
//         match map.get(&trigger_symbol) {
//             Some(combines) => combines.clone(),
//             None => return,
//         }
//     };

//     let mut instances = Vec::with_capacity(combines.len());
//     let mut handles = Vec::with_capacity(combines.len());

//     for combine in combines {
//         let symbols = [
//             combine.symbol0.clone(),
//             combine.symbol1.clone(),
//             combine.symbol2.clone(),
//         ];
//         let mut book_tickers = [None, None, None];
//         let mut fees = [None, None, None];
//         let mut min_notionals = [Decimal::zero(), Decimal::zero(), Decimal::zero()];
//         let mut lot_min_qtys = [Decimal::zero(), Decimal::zero(), Decimal::zero()];
//         let mut lot_step_sizes = [Decimal::zero(), Decimal::zero(), Decimal::zero()];
//         let mut lot_round_nums = [0, 0, 0];
//         let mut price_tick_sizes = [Decimal::zero(), Decimal::zero(), Decimal::zero()];
//         let mut price_round_nums = [0, 0, 0];

//         for (i, symbol) in symbols.iter().enumerate() {
//             if let Some(ticker) = get_book_ticker(symbol).await {
//                 if ticker.ask_qty.is_zero()
//                     || ticker.bid_qty.is_zero()
//                     || ticker.ask_price.is_zero()
//                     || ticker.bid_price.is_zero()
//                 {
//                     return;
//                 }
//                 book_tickers[i] = Some(ticker);
//             }

//             if let Some(fee) = get_fee(symbol).await {
//                 fees[i] = Some(fee);
//             } else {
//                 error(format!("未能获取到手续费费率. symbol: {}", symbol));
//                 return;
//             }

//             if let Some(filter) = get_filter(symbol).await {
//                 min_notionals[i] = filter.min_notional;
//                 lot_min_qtys[i] = filter.lot_min_qty;
//                 lot_step_sizes[i] = filter.lot_step_size;
//                 lot_round_nums[i] = filter.lot_round_num;
//                 price_tick_sizes[i] = filter.price_tick_size;
//                 price_round_nums[i] = filter.price_round_num;
//             } else {
//                 error(format!("未能获取到交易对Filters. symbol: {}", symbol));
//                 return;
//             }
//         }

//         let inst = Instance {
//             inst_id: format!("inst-{}-{}", combine.id, trigger_book_ticker.update_id),
//             arbitrage_combine: combine.clone(),
//             trigger_symbol: trigger_symbol.clone(),
//             trigger_book_ticker: trigger_book_ticker.clone(),
//             book_tickers: book_tickers.clone(),
//             fees: fees.clone(),
//             min_notionals: min_notionals.clone(),
//             lot_min_qtys: lot_min_qtys.clone(),
//             lot_step_sizes: lot_step_sizes.clone(),
//             lot_round_nums: lot_round_nums.clone(),
//             price_tick_sizes: price_tick_sizes.clone(),
//             price_round_nums: price_round_nums.clone(),
//         };

//         instances.push(inst.clone());
//         handles.push(tokio::spawn(async move {
//             inst.analysis_earnings().await;
//         }));
//     }

//     for handle in handles {
//         let _ = handle.await;
//     }

//     if instances.is_empty() {
//         return;
//     }

//     let mut best_inst = None;
//     let mut count_of_profit_instances = 0;

//     for inst in &instances {
//         if inst.planed_net_profit > Decimal::zero() {
//             count_of_profit_instances += 1;
//         }
//         if best_inst.is_none()
//             || inst.planed_net_profit > best_inst.as_ref().unwrap().planed_net_profit
//         {
//             best_inst = Some(inst.clone());
//         }
//     }

//     if let Some(inst) = best_inst {
//         if inst.planed_net_profit > Decimal::zero() {
//             info!(format!(
//                 "三角套利分析完成，计算耗时：{} 微秒。参与分析实例: {} 个，收益为正实例: {} 个",
//                 start_at.elapsed().as_micros(),
//                 instances.len(),
//                 count_of_profit_instances
//             ));

//             inst.analysis_done_time = Instant::now();
//             info!(format!(
//                 "计算耗时：{} 微秒。最佳实例: {:?}",
//                 start_at.elapsed().as_micros(),
//                 inst
//             ));

//             // ToDo: trigger symbol bookTicker unchanged才触发
//             triple_trade_in_pipeline(ctx, &inst).await;
//         }
//     }
// }
