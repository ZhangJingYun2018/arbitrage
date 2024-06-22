use arbitrage::{init, models};
use log::info;
use models::combinesmod;

#[tokio::main]
async fn main() {
    let pool = init::run().await;
    let _ = combinesmod::update_all_combines(pool).await;
    info!("组合更新完毕");
}
