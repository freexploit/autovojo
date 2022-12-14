pub mod autovojo_grpc {
    tonic::include_proto!("autovojo_control_plane");
}

use autovojo_common::Tunnel;
use aya::Bpf;
use autovojo_grpc::autovojo_control_plane_server::AutovojoControlPlane;
use autovojo_grpc::{Empty, AutovojoRequest, AutovojoResponse};
use aya::maps::HashMap;
use tokio::sync::Mutex;
//use std::env;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use autovojo_grpc::NodeDescriptor;

use self::autovojo_grpc::Nodes;

//use autovojo_grpc::NodeDescriptor;

// defining a struct for our service
//
struct Device  {
    id: String,
    ip_address: Ipv4Addr,
    //udp_port: i32,
    tcp_port: i32
}

pub struct AutovojoService {
    devices: Arc<Mutex<Vec<Device>>>,
    loader: Arc<Mutex<Bpf>>
}

impl AutovojoService {
    pub fn new(loader: Arc<Mutex<Bpf>>) -> AutovojoService {
        AutovojoService{
            devices : Arc::new(Mutex::new(Vec::new())),
            loader
        }
    }
}

// implementing rpc for service defined in .proto

#[tonic::async_trait]
impl AutovojoControlPlane for AutovojoService  {
    async fn register_node(&self,request: Request<AutovojoRequest>) -> Result<Response<AutovojoResponse>,Status> {
        let name = request.get_ref().node_name.clone();
        let ip_address= request.get_ref().ip.parse().unwrap();
        let tcp_port= request.get_ref().port;

        let device = Device {
            id: name,
            ip_address,
            tcp_port
        };
        let mut backend_devices: HashMap<_,u16,Tunnel>  =
            HashMap::try_from(self.loader.lock().await.map_mut("TUNNELS").unwrap()).unwrap();
            
        let back_device = Tunnel {
            ipv4_address: device.ip_address.clone().try_into().unwrap(),
            dst_port: u16::try_from(device.tcp_port.clone()).unwrap(),
            src_port: 0,
            src_ipv4_address:0,
        };

        match self.devices.try_lock() {
            Ok(mut d) => {
                backend_devices.insert(55555, back_device, 0).unwrap();
                d.push(device);
            },
            Err(e) => return Err(tonic::Status::internal(format!("Mutex error: {}",e.to_string())))
        };

        Ok(Response::new(AutovojoResponse{
            message: "New device added".into(),
            nodes: None
        }))
    }

    async fn remove_node(&self,_request: Request<AutovojoRequest>) -> Result<Response<AutovojoResponse>,Status> {
        todo!()
    }

    async fn list_nodes(&self,_request: Request<Empty>) -> Result<Response<AutovojoResponse>,Status> {

        let devices = self.devices.lock().await;


        let nodes = Nodes {
            nodes: devices.iter().map(|device |
            NodeDescriptor{
                node_name: device.id.clone(),
                node_ip: device.ip_address.to_string(),
                node_port: device.tcp_port,
            }
            ).collect::<Vec<NodeDescriptor>>()
        };


        Ok(Response::new(AutovojoResponse{
            message: "Devices".into(),
            nodes: Some(nodes)
        }))
    }
}
