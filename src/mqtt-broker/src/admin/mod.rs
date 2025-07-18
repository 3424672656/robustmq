// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod acl;
pub mod blacklist;
pub mod client;
pub mod cluster;
pub mod connector;
pub mod observability;
pub mod query;
pub mod schema;
pub mod session;
pub mod subscribe;
pub mod topic;
pub mod user;

use crate::handler::cache::CacheManager;
use crate::handler::flapping_detect::enable_flapping_detect;
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::manager::SubscribeManager;
use crate::{handler::error::MqttBrokerError, storage::cluster::ClusterStorage};

use common_base::tools::serialize_value;
use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::{
    BrokerNodeRaw, ClusterStatusReply, EnableFlappingDetectReply, EnableFlappingDetectRequest,
    ListConnectionRaw, ListConnectionReply,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn cluster_status_by_req(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
) -> Result<ClusterStatusReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

    let mut broker_node_list = Vec::new();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage.node_list().await?;
    for node in data {
        broker_node_list.push(format!("{}@{}", node.node_ip, node.node_id));
    }

    let placement_status = cluster_storage.place_cluster_status().await?;
    let node_list = cache_manager.node_list();
    let resp_node_list: Vec<BrokerNodeRaw> =
        node_list.iter().map(|node| node.clone().into()).collect();
    let reply = ClusterStatusReply {
        cluster_name: config.cluster_name.clone(),
        message_in_rate: 10,
        message_out_rate: 3,
        connection_num: connection_manager.connections.len() as u32,
        session_num: cache_manager.session_info.len() as u32,
        subscribe_num: subscribe_manager.subscribe_list.len() as u32,
        exclusive_subscribe_num: subscribe_manager.exclusive_push.len() as u32,
        exclusive_subscribe_thread_num: subscribe_manager.exclusive_push_thread.len() as u32,
        share_subscribe_leader_num: subscribe_manager.share_leader_push.len() as u32,
        share_subscribe_leader_thread_num: subscribe_manager.share_leader_push_thread.len() as u32,
        share_subscribe_resub_num: subscribe_manager.share_follower_resub.len() as u32,
        share_subscribe_follower_thread_num: subscribe_manager.share_follower_resub_thread.len()
            as u32,
        topic_num: cache_manager.topic_info.len() as u32,
        nodes: resp_node_list,
        placement_status,
        tcp_connection_num: connection_manager.tcp_write_list.len() as u32,
        tls_connection_num: connection_manager.tcp_tls_write_list.len() as u32,
        websocket_connection_num: connection_manager.websocket_write_list.len() as u32,
        quic_connection_num: connection_manager.quic_write_list.len() as u32,
    };
    let _ = subscribe_manager.snapshot_info();

    Ok(reply)
}

pub async fn enable_flapping_detect_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<EnableFlappingDetectRequest>,
) -> Result<Response<EnableFlappingDetectReply>, Status> {
    let req = request.into_inner();

    match enable_flapping_detect(client_pool, cache_manager, req).await {
        Ok(_) => Ok(Response::new(EnableFlappingDetectReply {
            is_enable: req.is_enable,
        })),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn list_connection_by_req(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
) -> Result<Response<ListConnectionReply>, Status> {
    let mut reply = ListConnectionReply::default();
    let mut list_connection_raw: Vec<ListConnectionRaw> = Vec::new();
    for (key, value) in connection_manager.list_connect() {
        if let Some(mqtt_value) = cache_manager.get_connection(key) {
            let mqtt_info = serialize_value(&mqtt_value)?;
            let raw = ListConnectionRaw {
                connection_id: value.connection_id,
                connection_type: value.connection_type.to_string(),
                protocol: match value.protocol {
                    Some(protocol) => protocol.into(),
                    None => "None".to_string(),
                },
                source_addr: value.addr.to_string(),
                info: mqtt_info,
            };
            list_connection_raw.push(raw);
        }
    }
    reply.list_connection_raw = list_connection_raw;
    Ok(Response::new(reply))
}
