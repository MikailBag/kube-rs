//! In this example we will implement something similar
//! to `kubectl get all --all-namespaces`.

use kube::{
    api::{Api, DynamicObject, ResourceExt},
    client::{Discovery, Scope},
    Client,
};
use log::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=debug");
    env_logger::init();
    let client = Client::try_default().await?;

    let v = client.apiserver_version().await?;
    info!("api version: {:?}", v);

    let ns_filter = std::env::var("NAMESPACE").ok();

    let discovery = Discovery::new(&client).await?;

    for group in discovery.groups() {
        let ver = group.preferred_version_or_guess();
        for (res, extras) in group.resources_by_version(ver) {
            let api: Api<DynamicObject> = if let Scope::Namespaced = extras.scope {
                if let Some(ns) = &ns_filter {
                    Api::namespaced_with(client.clone(), ns, &res)
                } else {
                    Api::all_with(client.clone(), &res)
                }
            } else {
                Api::all_with(client.clone(), &res)
            };

            info!("{}/{} : {}", group.name(), ver, res.kind);

            let list = match api.list(&Default::default()).await {
                Ok(l) => l,
                Err(e) => {
                    warn!("Failed to list: {:#}", e);
                    continue;
                }
            };
            for item in list.items {
                let name = item.name();
                let ns = item.metadata.namespace.map(|s| s + "/").unwrap_or_default();
                info!("\t\t{}{}", ns, name);
            }
        }
    }

    Ok(())
}
