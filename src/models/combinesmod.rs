use chrono::Utc;
use log::info;
use sqlx::MySqlPool;

use super::bean::arbitragecombine::ArbitrageCombine;
use super::bean::symbol::Symbol;

pub async fn update_all_combines(pool: MySqlPool) -> Result<(), sqlx::Error> {
    info!("开始三角套利组合计算");

    let db_combines = ArbitrageCombine::query_all_arbitrage_combines(&pool).await?;
    let all_symbols = Symbol::query_all_symbols(&pool).await?;
    let valid_symbols: Vec<Symbol> = all_symbols;

    info!(
        "当前套利组合：{}组；最新全部交易对：{}个（现货可交易的交易对：{}个）",
        db_combines.len(),
        valid_symbols.len(),
        valid_symbols.len()
    );
    let iterates: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    let start_at = Utc::now();

    let mut combines: Vec<ArbitrageCombine> = Vec::with_capacity(2000);
    let mut arbitrage_symbols: Vec<&String> = Vec::with_capacity(2000);
    let mut arbitrage_assets: Vec<&String> = Vec::with_capacity(2000);

    // 遍历所有交易对组合，获取有效的套利组合
    let valid_capital_assets = ["USDT", "BUSD"];
    // todo!("优化算法，减少重复计算"); 删除l 优化valid_symbols查询条件
    let l: usize = 500;
    for i in 0..l {
        for j in (i + 1)..l {
            for k in (j + 1)..l {
                let mut unique_assets = vec![
                    valid_symbols[i].get_base_asset().unwrap(),
                    valid_symbols[i].get_quote_asset().unwrap(),
                    valid_symbols[j].get_base_asset().unwrap(),
                    valid_symbols[j].get_quote_asset().unwrap(),
                    valid_symbols[k].get_base_asset().unwrap(),
                    valid_symbols[k].get_quote_asset().unwrap(),
                ];

                unique_string_slice(&mut unique_assets);
                if unique_assets.len() != 3 {
                    continue;
                }

                let mut symbol_names: Vec<&String> = vec![
                    valid_symbols[i].get_symbol().unwrap(),
                    valid_symbols[j].get_symbol().unwrap(),
                    valid_symbols[k].get_symbol().unwrap(),
                ];

                let symbols = [&valid_symbols[i], &valid_symbols[j], &valid_symbols[k]];

                let mut new_combines = false;
                iterates.iter().for_each(|n| {
                    let symbol0 = symbols[n[0]];
                    let symbol1 = symbols[n[1]];
                    let symbol2 = symbols[n[2]];

                    let capital_assets = vec![
                        symbol0.get_base_asset().unwrap(),
                        symbol0.get_quote_asset().unwrap(),
                    ];

                    for capital_asset in capital_assets {
                        if !valid_capital_assets.contains(&capital_asset.as_str()) {
                            continue;
                        }
                        // Your logic to calculate trade action and result asset
                        // if to_asset2 != capital_asset { continue; }
                        let (to_asset0, action0) = if let Some((to_asset0, action0)) =
                            calculate_trade_action_and_result_asset(
                                symbol0.get_base_asset().unwrap(),
                                symbol0.get_quote_asset().unwrap(),
                                capital_asset,
                            ) {
                            (to_asset0, action0)
                        } else {
                            continue;
                        };
                        let (to_asset1, action1) = if let Some((to_asset1, action1)) =
                            calculate_trade_action_and_result_asset(
                                symbol1.get_base_asset().unwrap(),
                                symbol1.get_quote_asset().unwrap(),
                                to_asset0,
                            ) {
                            (to_asset1, action1)
                        } else {
                            continue;
                        };
                        let (_, action2) = if let Some((to_asset2, action2)) =
                            calculate_trade_action_and_result_asset(
                                symbol2.get_base_asset().unwrap(),
                                symbol2.get_quote_asset().unwrap(),
                                to_asset1,
                            ) {
                            (to_asset2, action2)
                        } else {
                            continue;
                        };
                        combines.push(ArbitrageCombine {
                            id: None,
                            triple_assets: Some(format!(
                                "{},{},{}",
                                unique_assets[0], unique_assets[1], unique_assets[2],
                            )),
                            triple_symbols: Some(format!(
                                "{}, {}, {}",
                                symbol0.get_symbol().unwrap(),
                                symbol1.get_symbol().unwrap(),
                                symbol2.get_symbol().unwrap()
                            )),
                            asset0: Some(capital_asset.to_string()),
                            action0: Some(action0.to_string()),
                            symbol0: Some(symbol0.get_symbol().unwrap().to_string()),
                            asset1: Some(to_asset0.to_string()),
                            action1: Some(action1.to_string()),
                            symbol1: Some(symbol1.get_symbol().unwrap().to_string()),
                            asset2: Some(to_asset1.to_string()),
                            action2: Some(action2.to_string()),
                            symbol2: Some(symbol2.get_symbol().unwrap().to_string()),
                            hash: Some(format!(
                                "{}-{}-{}-{}-{}-{}-{}",
                                capital_asset,
                                action0,
                                symbol0.get_symbol().unwrap(),
                                action1,
                                symbol1.get_symbol().unwrap(),
                                action2,
                                symbol2.get_symbol().unwrap()
                            )),
                            created_at: Some(Utc::now().naive_utc()),
                            updated_at: Some(Utc::now().naive_utc()),
                        });
                        new_combines = true;
                    }
                });
                if new_combines {
                    arbitrage_symbols.append(&mut symbol_names);
                    arbitrage_assets.append(&mut unique_assets);
                }
            }
        }
    }
    unique_string_slice(&mut arbitrage_symbols);
    unique_string_slice(&mut arbitrage_assets);
    info!(
        "完成三角套利组合计算。可套利组合：{}组，涉及交易对：{}对，涉及币种：{}种，计算耗时：{}秒",
        combines.len(),
        arbitrage_symbols.len(),
        arbitrage_assets.len(),
        (Utc::now() - start_at).num_milliseconds()
    );

    //更新数据库
    let start_at = Utc::now();
    let mut delete_count = 0;

    let mut combines_map = std::collections::HashMap::new();
    for combine in &combines {
        if let Some(hash) = &combine.hash {
            combines_map.insert(hash.clone(), combine);
        }
    }

    for db_combine in db_combines {
        if let Some(hash) = db_combine.hash {
            if combines_map.contains_key(&hash) {
                delete_count += 1;
                let _ =
                    ArbitrageCombine::delete_arbitrage_combine(&pool, db_combine.id.unwrap()).await;
            }
        }
    }
    let v = ArbitrageCombine::insert_arbitrage_combines(&pool, combines).await?;
    info!(
        "完成三角套利组合DB存储。DB新增{}个,删除{}个,存储耗时: {}秒",
        v,
        delete_count,
        (Utc::now() - start_at).num_seconds()
    );
    Ok(())
}

fn unique_string_slice(slice: &mut Vec<&String>) {
    slice.sort();
    slice.dedup();
}

fn calculate_trade_action_and_result_asset<'a>(
    base_asset: &'a str,
    quote_asset: &'a str,
    from_asset: &str,
) -> Option<(&'a str, &'static str)> {
    if from_asset == base_asset {
        Some((quote_asset, "SELL"))
    } else if from_asset == quote_asset {
        Some((base_asset, "BUY"))
    } else {
        None
    }
}
