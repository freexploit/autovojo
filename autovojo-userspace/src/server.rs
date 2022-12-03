pub mod autovojo_grpc {
    tonic::include_proto!("autovojo_control_plane");
}

use autovojo_grpc::autovojo_control_plane_server::AutovojoControlPlane;
use autovojo_grpc::{Empty, AutovojoRequest, AutovojoResponse};
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
}

impl AutovojoService {
    pub fn new() -> AutovojoService {
        AutovojoService{
            devices : Arc::new(Mutex::new(Vec::new()))
        }
    }
}

// implementing rpc for service defined in .proto

#[tonic::async_trait]
impl AutovojoControlPlane for AutovojoService {
    async fn register_node(&self,request: Request<AutovojoRequest>) -> Result<Response<AutovojoResponse>,Status> {
        let name = request.get_ref().node_name.clone();
        let ip_address= request.get_ref().ip.parse().unwrap();
        let tcp_port= request.get_ref().port;

        let device = Device {
            id: name,
            ip_address,
            tcp_port
        };

        match self.devices.try_lock() {
            Ok(mut d) =>  d.push(device),
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
