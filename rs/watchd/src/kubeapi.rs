use std::error::Error;

use k8s_openapi::api::{
    apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet},
    core::v1::{Namespace, Node, Pod, Service, ServiceAccount},
    networking::v1::Ingress,
    rbac::v1::{ClusterRole, ClusterRoleBinding, Role},
    storage::v1::StorageClass,
};
use kube::Api;
use shared::client::Hik8sClient;

use crate::{constant::ROUTE_RESOURCE, watcher::setup_watcher};

#[derive(Clone)]
pub enum KubeApi {
    Deployment(Api<Deployment>),
    DaemonSet(Api<DaemonSet>),
    ReplicaSet(Api<ReplicaSet>),
    StatefulSet(Api<ReplicaSet>),
    Pod(Api<Pod>),
    Service(Api<Service>),
    Namespace(Api<Namespace>),
    Node(Api<Node>),
    Ingress(Api<Ingress>),
    ServiceAccount(Api<ServiceAccount>),
    Role(Api<Role>),
    ClusterRole(Api<ClusterRole>),
    ClusterRoleBinding(Api<ClusterRoleBinding>),
    StorageClass(Api<StorageClass>),
}

impl KubeApi {
    pub fn new_all(client: &kube::Client) -> Vec<Self> {
        vec![
            Self::Deployment(Api::all(client.clone())),
            Self::Pod(Api::all(client.clone())),
            Self::DaemonSet(Api::all(client.clone())),
            Self::ReplicaSet(Api::all(client.clone())),
            Self::StatefulSet(Api::all(client.clone())),
            Self::Service(Api::all(client.clone())),
            Self::Namespace(Api::all(client.clone())),
            Self::Node(Api::all(client.clone())),
            Self::Ingress(Api::all(client.clone())),
            Self::ServiceAccount(Api::all(client.clone())),
            Self::Role(Api::all(client.clone())),
            Self::ClusterRole(Api::all(client.clone())),
            Self::ClusterRoleBinding(Api::all(client.clone())),
            Self::StorageClass(Api::all(client.clone())),
        ]
    }

    pub async fn setup_watcher(self, client: Hik8sClient) -> Result<(), Box<dyn Error>> {
        let route = ROUTE_RESOURCE;
        match self {
            Self::Deployment(api) => setup_watcher(api, client, route, true).await,
            Self::DaemonSet(api) => setup_watcher(api, client, route, true).await,
            Self::ReplicaSet(api) => setup_watcher(api, client, route, true).await,
            Self::StatefulSet(api) => setup_watcher(api, client, route, true).await,
            Self::Pod(api) => setup_watcher(api, client, route, true).await,
            Self::Service(api) => setup_watcher(api, client, route, true).await,
            Self::Namespace(api) => setup_watcher(api, client, route, true).await,
            Self::Node(api) => setup_watcher(api, client, route, true).await,
            Self::Ingress(api) => setup_watcher(api, client, route, true).await,
            Self::ServiceAccount(api) => setup_watcher(api, client, route, true).await,
            Self::Role(api) => setup_watcher(api, client, route, true).await,
            Self::ClusterRole(api) => setup_watcher(api, client, route, true).await,
            Self::ClusterRoleBinding(api) => setup_watcher(api, client, route, true).await,
            Self::StorageClass(api) => setup_watcher(api, client, route, true).await,
        }
    }
}
