use std::net::SocketAddr;

use axum::{extract::ConnectInfo, routing::get, Router};
use ngrok::{config::TunnelBuilder, tunnel::{HttpTunnel, UrlTunnel}};

pub async fn server() -> anyhow::Result<()>{
    let app = Router::new().route(
        "/",
        get(
            |ConnectInfo(remote_addr): ConnectInfo<SocketAddr>| async move {
                format!("Hello, {remote_addr:?}!\r\n")
            },
        ),
    ).route(
        "/ls",
        get(
            |ConnectInfo(_remote_addr): ConnectInfo<SocketAddr>| async move {
                let paths = std::fs::read_dir("./").unwrap();
                let x = paths.into_iter().map(|x| x.map(|entry| entry.path())).collect::<Result<Vec<std::path::PathBuf>, _>>().unwrap();
                format!("Paths in local directory, {x:?}!\r\n")
            },
        ),
    );

    // !! this doesn't listen on any local ports (super cool)
    let tunnel = start_tunnel().await?;
    let _server_url = tunnel.url();
    axum::Server::builder(tunnel)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

async fn start_tunnel() -> anyhow::Result<HttpTunnel> {
    let tun = ngrok::Session::builder()
        .authtoken_from_env()
        .connect()
        .await?
        .http_endpoint()
        // .allow_cidr("0.0.0.0/0")
        // .basic_auth("ngrok", "online1line")
        // .circuit_breaker(0.5)
        // .compression()
        // .deny_cidr("10.1.1.1/32")
        // .verify_upstream_tls(false)
        // .domain("<somedomain>.ngrok.io")
        // .forwards_to("example rust")
        // .mutual_tlsca(CA_CERT.into())
        // .oauth(
        //     OauthOptions::new("google")
        //         .allow_email("<user>@<domain>")
        //         .allow_domain("<domain>")
        //         .scope("<scope>"),
        // )
        // .oidc(
        //     OidcOptions::new("<url>", "<id>", "<secret>")
        //         .allow_email("<user>@<domain>")
        //         .allow_domain("<domain>")
        //         .scope("<scope>"),
        // )
        // .policy(create_policy())?
        // .proxy_proto(ProxyProto::None)
        // .remove_request_header("X-Req-Nope")
        // .remove_response_header("X-Res-Nope")
        // .request_header("X-Req-Yup", "true")
        // .response_header("X-Res-Yup", "true")
        // .scheme(ngrok::Scheme::HTTPS)
        // .websocket_tcp_conversion()
        // .webhook_verification("twilio", "asdf"),
        .metadata("example tunnel metadata from rust")
        .listen()
        .await?;

    println!("\n\nTunnel started on URL: {:?}", tun.url());

    Ok(tun)
}