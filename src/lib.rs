#![allow(dead_code, unused_imports)]
pub mod apiclient;
pub mod handlers;
pub mod init;
pub mod models;
pub mod utils;
use axum::{routing::get, Extension, Router};
use hyper::Server;
use log::info;

use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
pub async fn run() {
    let pool = init::run().await;
    // 创建路由
    let app = Router::new()
        .route("/", get(handlers::handler))
        .layer(Extension(Arc::new(pool)));

    // 绑定地址并启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Listening on http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
