# Log daemon

This is a background program that reads log messages from the `/var/log/pods` directory and streams them to the Hik8s api endpoint.

## Release

```bash
VERSION=$(grep -E 'version = "[^"]*"$' rs/logd/Cargo.toml | awk -F\" '{print $2}') && echo $VERSION
docker build -t ghcr.io/hik8s/logd:$VERSION .
docker push ghcr.io/hik8s/logd:$VERSION
```
