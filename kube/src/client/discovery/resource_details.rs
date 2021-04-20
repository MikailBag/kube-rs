use crate::api::ApiResource;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::APIResourceList;

/// Resource scope
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Scope {
    /// Objects are global
    Cluster,
    /// Each object lives in namespace.
    Namespaced,
}

/// Operations that are supported on the resource
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Operations {
    /// Object can be created
    pub create: bool,
    /// Single object can be queried
    pub get: bool,
    /// Multiple objects can be queried
    pub list: bool,
    /// A watch can be started
    pub watch: bool,
    /// A single object can be deleted
    pub delete: bool,
    /// Multiple objects can be deleted
    pub delete_collection: bool,
    /// Object can be updated
    pub update: bool,
    /// Object can be patched
    pub patch: bool,
    /// All other verbs
    pub other: Vec<String>,
}

impl Operations {
    /// Returns empty `Operations`
    pub fn empty() -> Self {
        Operations {
            create: false,
            get: false,
            list: false,
            watch: false,
            delete: false,
            delete_collection: false,
            update: false,
            patch: false,
            other: Vec::new(),
        }
    }
}
/// Contains additional, detailed information abount API resource
#[derive(Debug, Clone)]
pub struct ApiResourceExtras {
    /// Scope of the resource
    pub scope: Scope,
    /// Available subresources. Please note that returned ApiResources are not
    /// standalone resources. Their name will be of form `subresource_name`,
    /// not `resource_name/subresource_name`.
    /// To work with subresources, use `Request` methods.
    pub subresources: Vec<(ApiResource, ApiResourceExtras)>,
    /// Supported operations on this resource
    pub operations: Operations,
}

impl ApiResourceExtras {
    /// Creates ApiResourceExtras from `meta::v1::APIResourceList` instance.
    /// This function correctly sets all fields except `subresources`.
    /// # Panics
    /// Panics if list does not contain resource named `name`.
    pub fn from_apiresourcelist(list: &APIResourceList, name: &str) -> Self {
        let ar = list
            .resources
            .iter()
            .find(|r| r.name == name)
            .expect("resource not found in APIResourceList");
        let scope = if ar.namespaced {
            Scope::Namespaced
        } else {
            Scope::Cluster
        };
        let mut operations = Operations::empty();
        for verb in &ar.verbs {
            match verb.as_str() {
                "create" => operations.create = true,
                "get" => operations.get = true,
                "list" => operations.list = true,
                "watch" => operations.watch = true,
                "delete" => operations.delete = true,
                "deletecollection" => operations.delete_collection = true,
                "update" => operations.update = true,
                "patch" => operations.patch = true,
                _ => operations.other.push(verb.clone()),
            }
        }
        let mut subresources = Vec::new();
        let subresource_name_prefix = format!("{}/", name);
        for res in &list.resources {
            if let Some(subresource_name) = res.name.strip_prefix(&subresource_name_prefix) {
                let mut api_resource = ApiResource::from_apiresource(res, &list.group_version);
                api_resource.plural = subresource_name.to_string();
                let extra = ApiResourceExtras::from_apiresourcelist(list, &res.name);
                subresources.push((api_resource, extra));
            }
        }

        ApiResourceExtras {
            scope,
            subresources,
            operations,
        }
    }
}
