use k8s_openapi::api::{
    apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet},
    core::v1::{Event, Namespace, Node, Pod, Service, ServiceAccount},
    networking::v1::Ingress,
    rbac::v1::{ClusterRole, ClusterRoleBinding, Role},
    storage::v1::StorageClass,
};
use kube::Api;
use shared::client::Hik8sClient;
use std::fmt;

use crate::{
    constant::{ROUTE_EVENT, ROUTE_RESOURCE},
    error::WatchDaemonError,
    watcher::setup_watcher,
};

#[derive(Clone)]
pub enum KubeApiResource {
    Event(Api<Event>),
    Deployment(Api<Deployment>),
    DaemonSet(Api<DaemonSet>),
    ReplicaSet(Api<ReplicaSet>),
    StatefulSet(Api<StatefulSet>),
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

impl fmt::Display for KubeApiResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Event(_) => "Event",
            Self::Deployment(_) => "Deployment",
            Self::DaemonSet(_) => "DaemonSet",
            Self::ReplicaSet(_) => "ReplicaSet",
            Self::StatefulSet(_) => "StatefulSet",
            Self::Pod(_) => "Pod",
            Self::Service(_) => "Service",
            Self::Namespace(_) => "Namespace",
            Self::Node(_) => "Node",
            Self::Ingress(_) => "Ingress",
            Self::ServiceAccount(_) => "ServiceAccount",
            Self::Role(_) => "Role",
            Self::ClusterRole(_) => "ClusterRole",
            Self::ClusterRoleBinding(_) => "ClusterRoleBinding",
            Self::StorageClass(_) => "StorageClass",
        };
        write!(f, "{}", name)
    }
}

impl KubeApiResource {
    pub fn new_all(client: &kube::Client) -> Vec<Self> {
        vec![
            Self::Event(Api::all(client.clone())),
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
    pub const fn route(&self) -> &'static str {
        match self {
            Self::Event(_) => ROUTE_EVENT,
            _ => ROUTE_RESOURCE,
        }
    }

    pub async fn setup_watcher(self, client: Hik8sClient) -> Result<(), WatchDaemonError> {
        let route = self.route();
        let name = self.to_string();
        match self {
            Self::Event(api) => setup_watcher(name, api, client, route, true).await,
            Self::Deployment(api) => setup_watcher(name, api, client, route, true).await,
            Self::DaemonSet(api) => setup_watcher(name, api, client, route, true).await,
            Self::ReplicaSet(api) => setup_watcher(name, api, client, route, true).await,
            Self::StatefulSet(api) => setup_watcher(name, api, client, route, true).await,
            Self::Pod(api) => setup_watcher(name, api, client, route, true).await,
            Self::Service(api) => setup_watcher(name, api, client, route, true).await,
            Self::Namespace(api) => setup_watcher(name, api, client, route, true).await,
            Self::Node(api) => setup_watcher(name, api, client, route, true).await,
            Self::Ingress(api) => setup_watcher(name, api, client, route, true).await,
            Self::ServiceAccount(api) => setup_watcher(name, api, client, route, true).await,
            Self::Role(api) => setup_watcher(name, api, client, route, true).await,
            Self::ClusterRole(api) => setup_watcher(name, api, client, route, true).await,
            Self::ClusterRoleBinding(api) => setup_watcher(name, api, client, route, true).await,
            Self::StorageClass(api) => setup_watcher(name, api, client, route, true).await,
        }
    }
}
