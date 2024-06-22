#[derive(Debug, Serialize, Deserialize)]
pub struct ArbitrageCombine {
    pub id: u32,
    pub triple_assets: String,     // 类型为 varchar(64)
    pub triple_symbols: String,    // 类型为 varchar(64)
    pub asset0: String,            // 起始资产，类型为 varchar(32)
    pub action0: String,           // 类型为 varchar(4)
    pub symbol0: String,           // 类型为 varchar(32)
    pub asset1: String,            // 第二交易所使用的资产，类型为 varchar(32)
    pub action1: String,           // 类型为 varchar(4)
    pub symbol1: String,           // 类型为 varchar(32)
    pub asset2: String,            // 第三交易所使用的资产，类型为 varchar(32)
    pub action2: String,           // 类型为 varchar(4)
    pub symbol2: String,           // 类型为 varchar(32)
    pub hash: String,              // 哈希值，类型为 varchar(128); unique index
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}

impl ArbitrageCombine {}
