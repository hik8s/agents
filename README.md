# Hik8s agents

This repository contains agents that need to be run in your cluster to run Hik8s:

- [rs/logd/README.md](./rs/logd/README.md) (log daemon)

## Local development

The code in this program is specific to Linux and requires a development container to be compiled on macOS. To do this, we create a Docker image and run a container from this image in which the relevant files are mounted as volumes:

1. Build the image

    ```bash
    IMAGE_NAME="hik8s/agents-dev-container"
    docker build -t $IMAGE_NAME -f Dockerfile.dev .
    ```

2. Run a container

    ```bash
    IMAGE_NAME="hik8s/agents-dev-container"
    WORKDIR=$(grep 'WORKDIR' Dockerfile.dev | awk '{print $2}') && echo $WORKDIR

    docker run -it \
    --volume "$(pwd)/rs:$WORKDIR/rs" \
    --volume "$(pwd)/Cargo.toml:$WORKDIR/Cargo.toml" \
    --volume "$(pwd)/.env:$WORKDIR/.env" \
    $IMAGE_NAME
    ```

3. Inside the container, run:

    ```bash
    cargo watch -x run
    ```

This runs logd and recompiles when you make changes in your IDE.

## System diagram

The main components are log-daemon and wathch-daemon. This is how they interact with your Kubernetes cluster:

```mermaid
graph TB
    subgraph "Kubernetes Cluster"
        subgraph "Node 1"
            LD1[logd]
            FS1[(Host Filesystem)]
            LD1 -->|reads| FS1
        end
        
        subgraph "Node 2"
            LD2[logd]
            FS2[(Host Filesystem)]
            LD2 -->|reads /var/log/pods/*| FS2
        end

        subgraph "Control Plane"
            API[kube-apiserver]
            WD[watchd]
            WD -->|watches resources| API
            WD -->|watches events| API
            WD -->|watches CRDs| API
        end
    end

    HK[api.hik8s.ai]
    LD1 -->|sends logs| HK
    LD2 -->|sends logs| HK
    WD -->|sends events & manifests| HK

    classDef daemon fill:#e1bee7,stroke:#8e24aa
    classDef api fill:#bbdefb,stroke:#1976d2
    classDef backend fill:#c8e6c9,stroke:#388e3c
    classDef storage fill:#fff3e0,stroke:#f57c00

    class LD1,LD2 daemon
    class API api
    class HK backend
    class FS1,FS2 storage
```
