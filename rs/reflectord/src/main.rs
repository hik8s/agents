use kube::Client;
use reflectord::customresource::list_all_custom_resources;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::try_default().await?;

    // Setup Pod watcher
    // setup_watcher::<Pod>(client.clone()).await?;
    // setup_watcher::<CustomResourceDefinition>(client.clone()).await?;

    list_all_custom_resources(client).await?;
    // list_crds(client.clone()).await?;
    // Keep the main thread alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
