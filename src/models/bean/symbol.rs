use std::sync::Arc;

use axum::Extension;
use chrono::NaiveDateTime;
use sqlx::{FromRow, MySql, MySqlPool, Pool};

#[derive(Debug, FromRow, Clone)]
pub struct Symbol {
    pub id: u64,                                    // 主键ID
    pub exchange: Option<String>,                   // 交易所
    pub symbol: Option<String>,                     // 交易对
    pub status: Option<String>,                     // 状态
    base_asset: Option<String>,                     // 基础资产
    quote_asset: Option<String>,                    // 报价资产
    pub base_asset_precision: Option<i32>,          // 基础资产精度
    pub quote_asset_precision: Option<i32>,         // 报价资产精度
    pub base_commission_precision: Option<i32>,     // 基础佣金精度
    pub quote_commission_precision: Option<i32>,    // 报价佣金精度
    pub order_types: Option<String>,                // 订单类型
    pub iceberg_allowed: Option<i8>,                // 是否允许冰山订单
    pub oco_allowed: Option<i8>,                    // 是否允许oco订单
    pub quote_order_qty_market_allowed: Option<i8>, // 是否允许市价订单
    pub allow_trailing_stop: Option<i8>,            // 是否允许跟踪止损
    pub is_spot_trading_allowed: Option<i8>,        // 是否允许现货交易
    pub is_margin_trading_allowed: Option<i8>,      // 是否允许保证金交易
    pub filters: Option<String>,                    // 过滤器
    pub permissions: Option<String>,                // 权限
    pub created_at: Option<NaiveDateTime>,          // 创建时间
    pub updated_at: Option<NaiveDateTime>,          // 更新时间
}

impl Symbol {
    pub async fn get_symbols(Extension(pool): Extension<Arc<Pool<MySql>>>) -> Vec<Self> {
        sqlx::query_as!(Symbol, "SELECT * FROM symbols")
            .fetch_all(&*pool)
            .await
            .expect("Failed to fetch users")
    }
    pub async fn query_all_symbols(pool: &MySqlPool) -> Result<Vec<Symbol>, sqlx::Error> {
        sqlx::query_as!(Symbol, "SELECT * FROM symbols  ")
            .fetch_all(pool)
            .await
    }
}

impl Symbol {
    pub fn get_base_asset(&self) -> Option<&String> {
        self.base_asset.as_ref()
    }
    pub fn get_quote_asset(&self) -> Option<&String> {
        self.quote_asset.as_ref()
    }
    pub fn get_symbol(&self) -> Option<&String> {
        self.symbol.as_ref()
    }
}
