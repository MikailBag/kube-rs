#[macro_use] extern crate log;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Client, Tls, api::{Api, ListParams}};
use kube_runtime::watcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=debug");
    env_logger::init();
    let client = Client::try_default(Tls::pick()).await?;
    let namespace = std::env::var("NAMESPACE").unwrap_or("default".into());

    let cms: Api<ConfigMap> = Api::namespaced(client, &namespace);
    let lp = ListParams::default().allow_bookmarks();

    let mut w = watcher(cms, lp).boxed();
    while let Some(event) = w.try_next().await? {
        info!("Got: {:?}", event);
    }
    Ok(())
}
