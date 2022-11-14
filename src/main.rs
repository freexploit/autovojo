pub mod server;
//use std::path::Path;
use anyhow::Result;
use clap::Parser;
use ipnet::Ipv4Net;
use server::AutovojoService;
use server::autovojo_grpc::autovojo_control_plane_server::AutovojoControlPlaneServer;
use tonic::transport::Server;


#[derive(Parser,Debug)]
#[clap(author,version,about,long_about=None)]
struct Args {
    #[clap(short='b',long, default_value="/usr/bin/nebula")]
    nebula_path: String,
    #[clap(short='o',long,default_value="ca.crt")]
    org_ca: String,
    #[clap(short='k',long,default_value="ca.key")]
    org_key: String,
    #[clap(short='n',long)]
    db_path: Option<String>,
    #[clap(short='c',long,default_value="172.1.100.0/16")]
    network_cidr: Ipv4Net,
    #[clap(short='p',long)]
    s3_path: Option<String>,
    #[clap(short='s',long)]
    s3_bucket: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {

    std::env::set_var("RUST_LOG", "debug=debug");

    env_logger::init();

    let _args = Args::parse();

    let addr: std::net::SocketAddr = "0.0.0.0:50051".parse().unwrap();

    log::info!("listening into:{}", addr.to_string());

    //let mut plane_params = PlaneParams::new();

    let autovojo_service = AutovojoService::new();


    Server::builder()
        .add_service(AutovojoControlPlaneServer::new(autovojo_service))
        .serve(addr)
        .await?;

    Ok(())
}

