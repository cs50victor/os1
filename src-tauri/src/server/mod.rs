mod system_messages;
mod tunnel;

use std::net::SocketAddr;

use axum::{extract::ConnectInfo, routing::get, Router};
use ngrok::{
    config::TunnelBuilder,
    tunnel::{HttpTunnel, UrlTunnel},
};
use parking_lot::Mutex;

pub async fn server(tunnel : HttpTunnel) -> anyhow::Result<()> {
    let app = Router::new()
        .route(
            "/",
            get(|ConnectInfo(remote_addr): ConnectInfo<SocketAddr>| async move {
                format!("Hello, {remote_addr:?}!\r\n")
            }),
        )
        .route(
            "/ls",
            get(|ConnectInfo(_remote_addr): ConnectInfo<SocketAddr>| async move {
                let paths = std::fs::read_dir("./").unwrap();
                let x = paths
                    .into_iter()
                    .map(|x| x.map(|entry| entry.path()))
                    .collect::<Result<Vec<std::path::PathBuf>, _>>()
                    .unwrap();
                format!("Paths in local directory, {x:?}!\r\n")
            }),
        );

    // !! this doesn't listen on any local ports (super cool)
    println!(">> {}", tunnel.url().to_string());

    axum::Server::builder(tunnel)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

