pub mod server;

use std::net::{SocketAddr, self};
use std::sync::Arc;
use autovojo_common::PacketLog;
use aya::maps::perf::AsyncPerfEventArray;
use aya::{include_bytes_aligned, Bpf};
use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};
use aya::util::online_cpus;
use aya_log::BpfLogger;
use log::{info, warn};
use tokio::sync::Mutex;
use tokio::{signal, task};
use bytes::BytesMut;

//use std::path::Path;
use anyhow::Result;
use clap::Parser;
use server::AutovojoService;
use server::autovojo_grpc::autovojo_control_plane_server::AutovojoControlPlaneServer;
use tonic::transport::Server;


#[derive(Parser,Debug)]
#[clap(author,version,about,long_about=None)]
struct Args {
    #[clap(short,long,default_value="0.0.0.0")]
    bind_address: String,
    #[clap(short,long,default_value="50051")]
    port: u16,
    #[clap(short, long, default_value = "enp36s0")]
    iface: String,
}

#[tokio::main]
async fn main() -> Result<()> {

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let args = Args::parse();


    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/autovojo"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/autovojo"
    ))?;
    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }


    let dir = std::env::current_dir().unwrap();
    println!("{:?}", dir);


    let program: &mut Xdp = bpf.program_mut("xdp").unwrap().try_into()?;
    program.load()?;
    program.attach(&args.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    let arc_bpf = Arc::new(Mutex::new(bpf));
    let perf = arc_bpf.clone();

    task::spawn(async move {

        let addr: SocketAddr = format!("{}:{}",args.bind_address, args.port).parse().unwrap();
        info!("listening into:{}", addr.to_string());
        let autovojo_service = AutovojoService::new(arc_bpf);

        Server::builder()
            .add_service(AutovojoControlPlaneServer::new(autovojo_service))
            .serve(addr)
        .await.unwrap();
    });

    let mut perf_array = AsyncPerfEventArray::try_from(perf.lock().await.map_mut("EVENTS")?)?;


    for cpu_id in online_cpus()? {
        // 

        let mut buf = perf_array.open(cpu_id, None)?;

        task::spawn(async move {
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(1024))
                .collect::<Vec<_>>();
            loop {
                let events = buf.read_events(&mut buffers).await.unwrap();
                for i in 0..events.read {
                    let buf = &mut buffers[i];
                    let ptr = buf.as_ptr() as *const PacketLog;
                    // 

                    let data = unsafe { ptr.read_unaligned() };
                    let src_addr = net::Ipv4Addr::from(data.ipv4_address);
                    let src_port = data.src_port;
                    let dst_port = data.dst_port;
                    info!("LOG: SRC {}, ACTION {}, SRC PORT {}, DST PORT {}", src_addr, data.action, src_port, dst_port);

                }
            }
        });
    }


    log::info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
