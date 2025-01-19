#[derive(Debug, thiserror::Error)]
pub enum WatchDaemonError {
    #[error("Kubernetes client error: {0}")]
    KubeError(#[from] kube::Error),
}
