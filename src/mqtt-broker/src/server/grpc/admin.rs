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

use crate::admin::acl::{create_acl_by_req, delete_acl_by_req, list_acl_by_req};
use crate::admin::blacklist::{
    create_blacklist_by_req, delete_blacklist_by_req, list_blacklist_by_req,
};
use crate::admin::client::list_client_by_req;
use crate::admin::cluster::set_cluster_config_by_req;
use crate::admin::connector::{
    create_connector_by_req, delete_connector_by_req, list_connector_by_req,
    update_connector_by_req,
};
use crate::admin::observability::{
    list_slow_subscribe_by_req, list_system_alarm_by_req, set_system_alarm_config_by_req,
};
use crate::admin::schema::{
    bind_schema_by_req, create_schema_by_req, delete_schema_by_req, list_bind_schema_by_req,
    list_schema_by_req, unbind_schema_by_req, update_schema_by_req,
};
use crate::admin::session::list_session_by_req;
use crate::admin::subscribe::{
    delete_auto_subscribe_rule, list_auto_subscribe_rule_by_req, set_auto_subscribe_rule,
};
use crate::admin::topic::{
    create_topic_rewrite_rule_by_req, delete_topic_rewrite_rule_by_req,
    get_all_topic_rewrite_rule_by_req, list_topic_by_req,
};
use crate::admin::user::{create_user_by_req, delete_user_by_req, list_user_by_req};
use crate::admin::{cluster_status_by_req, enable_flapping_detect_by_req, list_connection_by_req};
use crate::handler::cache::CacheManager;
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::manager::SubscribeManager;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminService;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest, DeleteAclReply,
    DeleteAclRequest, DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest,
    DeleteBlacklistReply, DeleteBlacklistRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, EnableFlappingDetectReply,
    EnableFlappingDetectRequest, GetClusterConfigReply, GetClusterConfigRequest, ListAclReply,
    ListAclRequest, ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest, ListBlacklistReply,
    ListBlacklistRequest, ListClientReply, ListClientRequest, ListConnectionReply,
    ListConnectionRequest, ListRewriteTopicRuleReply, ListRewriteTopicRuleRequest,
    ListSessionReply, ListSessionRequest, ListSlowSubscribeReply, ListSlowSubscribeRequest,
    ListSystemAlarmReply, ListSystemAlarmRequest, ListTopicReply, ListTopicRequest, ListUserReply,
    ListUserRequest, MqttBindSchemaReply, MqttBindSchemaRequest, MqttCreateConnectorReply,
    MqttCreateConnectorRequest, MqttCreateSchemaReply, MqttCreateSchemaRequest,
    MqttDeleteConnectorReply, MqttDeleteConnectorRequest, MqttDeleteSchemaReply,
    MqttDeleteSchemaRequest, MqttListBindSchemaReply, MqttListBindSchemaRequest,
    MqttListConnectorReply, MqttListConnectorRequest, MqttListSchemaReply, MqttListSchemaRequest,
    MqttUnbindSchemaReply, MqttUnbindSchemaRequest, MqttUpdateConnectorReply,
    MqttUpdateConnectorRequest, MqttUpdateSchemaReply, MqttUpdateSchemaRequest,
    SetAutoSubscribeRuleReply, SetAutoSubscribeRuleRequest, SetClusterConfigReply,
    SetClusterConfigRequest, SetSystemAlarmConfigReply, SetSystemAlarmConfigRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcAdminServices {
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    subscribe_manager: Arc<SubscribeManager>,
}

impl GrpcAdminServices {
    pub fn new(
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
        connection_manager: Arc<ConnectionManager>,
        subscribe_manager: Arc<SubscribeManager>,
    ) -> Self {
        GrpcAdminServices {
            client_pool,
            cache_manager,
            connection_manager,
            subscribe_manager,
        }
    }
}

#[tonic::async_trait]
impl MqttBrokerAdminService for GrpcAdminServices {
    async fn mqtt_broker_set_cluster_config(
        &self,
        request: Request<SetClusterConfigRequest>,
    ) -> Result<Response<SetClusterConfigReply>, Status> {
        let request = request.into_inner().clone();
        set_cluster_config_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SetClusterConfigReply {
            feature_name: request.feature_name.clone(),
            is_enable: true,
        }))
    }

    async fn mqtt_broker_get_cluster_config(
        &self,
        _request: Request<GetClusterConfigRequest>,
    ) -> Result<Response<GetClusterConfigReply>, Status> {
        Ok(Response::new(GetClusterConfigReply {
            mqtt_broker_cluster_dynamic_config: serde_json::to_vec(
                &self.cache_manager.get_cluster_config(),
            )
            .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    // --- cluster ---
    async fn cluster_status(
        &self,
        _: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        match cluster_status_by_req(
            &self.client_pool,
            &self.subscribe_manager,
            &self.connection_manager,
            &self.cache_manager,
        )
        .await
        {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    // --- user ---
    async fn mqtt_broker_create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        create_user_by_req(&self.cache_manager, &self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateUserReply {}))
    }

    async fn mqtt_broker_delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        delete_user_by_req(&self.cache_manager, &self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteUserReply {}))
    }

    async fn mqtt_broker_list_user(
        &self,
        request: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let (users, count) = list_user_by_req(&self.cache_manager, &self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListUserReply {
            users,
            total_count: count as u32,
        }))
    }

    async fn mqtt_broker_list_client(
        &self,
        request: Request<ListClientRequest>,
    ) -> Result<Response<ListClientReply>, Status> {
        let (clients, count) = list_client_by_req(&self.cache_manager, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListClientReply {
            clients,
            total_count: count as u32,
        }))
    }

    async fn mqtt_broker_list_session(
        &self,
        request: Request<ListSessionRequest>,
    ) -> Result<Response<ListSessionReply>, Status> {
        let (sessions, count) = list_session_by_req(&self.cache_manager, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListSessionReply {
            sessions,
            total_count: count as u32,
        }))
    }

    async fn mqtt_broker_list_acl(
        &self,
        _: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        let acls = list_acl_by_req(&self.cache_manager, &self.client_pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListAclReply {
            acls: acls.clone(),
            total_count: acls.len() as u32,
        }))
    }

    async fn mqtt_broker_create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        create_acl_by_req(&self.cache_manager, &self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateAclReply {}))
    }

    async fn mqtt_broker_delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        delete_acl_by_req(&self.cache_manager, &self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteAclReply {}))
    }

    async fn mqtt_broker_list_blacklist(
        &self,
        request: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        let (blacklists, count) =
            list_blacklist_by_req(&self.cache_manager, &self.client_pool, request)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListBlacklistReply {
            blacklists,
            total_count: count as u32,
        }))
    }

    async fn mqtt_broker_delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        delete_blacklist_by_req(&self.cache_manager, &self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteBlacklistReply {}))
    }

    async fn mqtt_broker_create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        create_blacklist_by_req(&self.cache_manager, &self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateBlacklistReply {}))
    }

    async fn mqtt_broker_enable_flapping_detect(
        &self,
        request: Request<EnableFlappingDetectRequest>,
    ) -> Result<Response<EnableFlappingDetectReply>, Status> {
        enable_flapping_detect_by_req(&self.client_pool, &self.cache_manager, request).await
    }

    async fn mqtt_broker_set_system_alarm_config(
        &self,
        request: Request<SetSystemAlarmConfigRequest>,
    ) -> Result<Response<SetSystemAlarmConfigReply>, Status> {
        let req = request.into_inner();
        set_system_alarm_config_by_req(&self.cache_manager, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn mqtt_broker_list_system_alarm(
        &self,
        request: Request<ListSystemAlarmRequest>,
    ) -> Result<Response<ListSystemAlarmReply>, Status> {
        let req = request.into_inner();
        list_system_alarm_by_req(&self.cache_manager, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // --- connection ---
    async fn mqtt_broker_list_connection(
        &self,
        _: Request<ListConnectionRequest>,
    ) -> Result<Response<ListConnectionReply>, Status> {
        list_connection_by_req(&self.connection_manager, &self.cache_manager).await
    }

    async fn mqtt_broker_list_slow_subscribe(
        &self,
        request: Request<ListSlowSubscribeRequest>,
    ) -> Result<Response<ListSlowSubscribeReply>, Status> {
        list_slow_subscribe_by_req(&self.cache_manager, request).await
    }

    async fn mqtt_broker_list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        let (topics, count) = list_topic_by_req(&self.cache_manager, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListTopicReply {
            topics,
            total_count: count as u32,
        }))
    }

    async fn mqtt_broker_delete_topic_rewrite_rule(
        &self,
        request: Request<DeleteTopicRewriteRuleRequest>,
    ) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
        delete_topic_rewrite_rule_by_req(&self.client_pool, &self.cache_manager, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteTopicRewriteRuleReply {}))
    }

    async fn mqtt_broker_create_topic_rewrite_rule(
        &self,
        request: Request<CreateTopicRewriteRuleRequest>,
    ) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
        create_topic_rewrite_rule_by_req(&self.client_pool, &self.cache_manager, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateTopicRewriteRuleReply {}))
    }

    async fn mqtt_broker_get_all_topic_rewrite_rule(
        &self,
        _request: Request<ListRewriteTopicRuleRequest>,
    ) -> Result<Response<ListRewriteTopicRuleReply>, Status> {
        let rewrite_topic_rules = get_all_topic_rewrite_rule_by_req(&self.cache_manager)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let total_count = rewrite_topic_rules.len() as u32;
        Ok(Response::new(ListRewriteTopicRuleReply {
            rewrite_topic_rules,
            total_count,
        }))
    }

    async fn mqtt_broker_list_connector(
        &self,
        request: Request<MqttListConnectorRequest>,
    ) -> Result<Response<MqttListConnectorReply>, Status> {
        let connectors = list_connector_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttListConnectorReply { connectors }))
    }

    async fn mqtt_broker_create_connector(
        &self,
        request: Request<MqttCreateConnectorRequest>,
    ) -> Result<Response<MqttCreateConnectorReply>, Status> {
        create_connector_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttCreateConnectorReply {}))
    }

    async fn mqtt_broker_delete_connector(
        &self,
        request: Request<MqttDeleteConnectorRequest>,
    ) -> Result<Response<MqttDeleteConnectorReply>, Status> {
        delete_connector_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttDeleteConnectorReply {}))
    }

    async fn mqtt_broker_update_connector(
        &self,
        request: Request<MqttUpdateConnectorRequest>,
    ) -> Result<Response<MqttUpdateConnectorReply>, Status> {
        update_connector_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttUpdateConnectorReply {}))
    }

    // --- schema ---
    async fn mqtt_broker_list_schema(
        &self,
        request: Request<MqttListSchemaRequest>,
    ) -> Result<Response<MqttListSchemaReply>, Status> {
        let schemas = list_schema_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttListSchemaReply { schemas }))
    }

    async fn mqtt_broker_create_schema(
        &self,
        request: Request<MqttCreateSchemaRequest>,
    ) -> Result<Response<MqttCreateSchemaReply>, Status> {
        create_schema_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttCreateSchemaReply {}))
    }

    async fn mqtt_broker_update_schema(
        &self,
        request: Request<MqttUpdateSchemaRequest>,
    ) -> Result<Response<MqttUpdateSchemaReply>, Status> {
        update_schema_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttUpdateSchemaReply {}))
    }

    async fn mqtt_broker_delete_schema(
        &self,
        request: Request<MqttDeleteSchemaRequest>,
    ) -> Result<Response<MqttDeleteSchemaReply>, Status> {
        delete_schema_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttDeleteSchemaReply {}))
    }

    async fn mqtt_broker_list_bind_schema(
        &self,
        request: Request<MqttListBindSchemaRequest>,
    ) -> Result<Response<MqttListBindSchemaReply>, Status> {
        let schema_binds = list_bind_schema_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttListBindSchemaReply { schema_binds }))
    }

    async fn mqtt_broker_bind_schema(
        &self,
        request: Request<MqttBindSchemaRequest>,
    ) -> Result<Response<MqttBindSchemaReply>, Status> {
        bind_schema_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttBindSchemaReply {}))
    }

    async fn mqtt_broker_unbind_schema(
        &self,
        request: Request<MqttUnbindSchemaRequest>,
    ) -> Result<Response<MqttUnbindSchemaReply>, Status> {
        unbind_schema_by_req(&self.client_pool, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MqttUnbindSchemaReply {}))
    }

    async fn mqtt_broker_set_auto_subscribe_rule(
        &self,
        request: Request<SetAutoSubscribeRuleRequest>,
    ) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
        set_auto_subscribe_rule(&self.client_pool, &self.cache_manager, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SetAutoSubscribeRuleReply {}))
    }

    async fn mqtt_broker_delete_auto_subscribe_rule(
        &self,
        request: Request<DeleteAutoSubscribeRuleRequest>,
    ) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
        delete_auto_subscribe_rule(&self.client_pool, &self.cache_manager, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteAutoSubscribeRuleReply {}))
    }

    async fn mqtt_broker_list_auto_subscribe_rule(
        &self,
        _request: Request<ListAutoSubscribeRuleRequest>,
    ) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
        let auto_subscribe_rules = list_auto_subscribe_rule_by_req(&self.cache_manager)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListAutoSubscribeRuleReply {
            auto_subscribe_rules,
        }))
    }
}
