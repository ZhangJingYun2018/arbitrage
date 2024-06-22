use std::sync::Arc;

use axum::Extension;
use chrono::{NaiveDateTime, Utc};
use sqlx::{FromRow, MySql, MySqlPool, Pool};

#[derive(FromRow, Clone, Debug)]
pub struct ArbitrageCombine {
    pub id: Option<u64>,                   // 自增 id，类型为 u64,
    pub triple_assets: Option<String>,     // 类型为 varchar(64)
    pub triple_symbols: Option<String>,    // 类型为 varchar(64)
    pub asset0: Option<String>,            // 起始资产，类型为 varchar(32)
    pub action0: Option<String>,           // 类型为 varchar(4)
    pub symbol0: Option<String>,           // 类型为 varchar(32)
    pub asset1: Option<String>,            // 第二交易所使用的资产，类型为 varchar(32)
    pub action1: Option<String>,           // 类型为 varchar(4)
    pub symbol1: Option<String>,           // 类型为 varchar(32)
    pub asset2: Option<String>,            // 第三交易所使用的资产，类型为 varchar(32)
    pub action2: Option<String>,           // 类型为 varchar(4)
    pub symbol2: Option<String>,           // 类型为 varchar(32)
    pub hash: Option<String>,              // 哈希值，类型为 varchar(128); unique index
    pub created_at: Option<NaiveDateTime>, // 创建时间
    pub updated_at: Option<NaiveDateTime>, // 更新时间
}

impl ArbitrageCombine {
    pub async fn get_arbitrage_combines(
        Extension(pool): Extension<Arc<Pool<MySql>>>,
    ) -> Vec<ArbitrageCombine> {
        sqlx::query_as!(ArbitrageCombine, "SELECT id, triple_assets, triple_symbols, asset0, action0, symbol0, asset1, action1, symbol1, asset2, action2, symbol2, hash, created_at, updated_at FROM arbitrage_combines")
            .fetch_all(&*pool)
            .await
            .expect("Failed to fetch users")
    }

    pub async fn query_all_arbitrage_combines(
        pool: &MySqlPool,
    ) -> Result<Vec<ArbitrageCombine>, sqlx::Error> {
        sqlx::query_as!(ArbitrageCombine, "SELECT * FROM arbitrage_combines")
            .fetch_all(pool)
            .await
    }

    pub async fn insert_arbitrage_combines(
        pool: &MySqlPool,
        arbitrage_combines: Vec<ArbitrageCombine>,
    ) -> Result<u64, sqlx::Error> {
        let mut tx = pool.begin().await.unwrap();
        for combine in &arbitrage_combines {
            sqlx::query("INSERT INTO arbitrage_combines (triple_assets, triple_symbols, asset0, action0, symbol0, asset1, action1, symbol1, asset2, action2, symbol2, hash, created_at, updated_at)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                             ON DUPLICATE KEY 
                             UPDATE 
                             triple_assets = ?, triple_symbols = ?, asset0 = ?, action0 = ?, symbol0 = ?, asset1 = ?, action1 = ?, symbol1 = ?, asset2 = ?, action2 = ?, symbol2 = ?, updated_at = ?")
                .bind(combine.triple_assets.clone())
                .bind(combine.triple_symbols.clone())
                .bind(combine.asset0.clone())
                .bind(combine.action0.clone())
                .bind(combine.symbol0.clone())
                .bind(combine.asset1.clone())
                .bind(combine.action1.clone())
                .bind(combine.symbol1.clone())
                .bind(combine.asset2.clone())
                .bind(combine.action2.clone())
                .bind(combine.symbol2.clone())
                .bind(combine.hash.clone())
                .bind(combine.created_at)
                .bind(Utc::now().naive_utc())
                .bind(combine.triple_assets.clone())
                .bind(combine.triple_symbols.clone())
                .bind(combine.asset0.clone())
                .bind(combine.action0.clone())
                .bind(combine.symbol0.clone())
                .bind(combine.asset1.clone())
                .bind(combine.action1.clone())
                .bind(combine.symbol1.clone())
                .bind(combine.asset2.clone())
                .bind(combine.action2.clone())
                .bind(combine.symbol2.clone())
                .bind(Utc::now().naive_utc())
             .execute(&mut *tx)
            .await?;
        }
        tx.commit().await.unwrap();
        Ok(arbitrage_combines.len() as u64)
    }

    pub async fn delete_arbitrage_combine(pool: &MySqlPool, id: u64) -> Result<u64, sqlx::Error> {
        let r = sqlx::query("DELETE FROM arbitrage_combine WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(r.rows_affected() as u64)
    }
}
