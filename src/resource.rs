use crate::core::Config;
use crate::core::Identifier;
use crate::core::Version;
use crate::step::Get;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ResourceType {
    name: Identifier,
    #[serde(rename(serialize = "type"))]
    type_: Identifier,
    source: Config,
}

impl ResourceType {
    pub fn name(&self) -> Identifier {
        self.name.clone()
    }

    pub fn type_(&self) -> Identifier {
        self.type_.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Resource {
    name: Identifier,
    type_: ResourceType,
    source: Config,
}

impl Serialize for Resource {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Resource", 3)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("type", &self.type_.name)?;
        state.serialize_field("source", &self.source)?;
        state.end()
    }
}

impl Resource {
    pub fn new(name: &str, res_type: &ResourceType) -> Self {
        Self {
            name: name.to_string(),
            type_: res_type.clone(),
            source: HashMap::new(),
        }
    }

    pub fn name(&self) -> Identifier {
        self.name.clone()
    }

    pub fn resource_type(&self) -> &ResourceType {
        &self.type_
    }

    pub fn with_source(mut self, source: &Vec<(&str, &str)>) -> Self {
        source
            .iter()
            .map(|(k, v)| self.source.insert(k.to_string(), v.to_string()))
            // Iterators are lazy, use .count() to evaluate it.
            .count();
        self
    }

    pub fn as_get_resource(&self) -> Get {
        Get::from("", self, None)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AnonymousResource {
    #[serde(rename(serialize = "type"))]
    type_: Identifier,
    source: Config,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Config>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<Version>,
}

impl AnonymousResource {
    pub fn from(type_: &str, source: &Vec<(&str, &str)>) -> Self {
        Self {
            type_: type_.to_string(),
            source: source
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            params: None,
            version: None,
        }
    }

    pub fn with_params(&self, params: &Vec<(&str, &str)>) -> Self {
        let mut this = self.clone();
        this.params = Some(
            params
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        );
        this
    }

    pub fn with_version(&self, version: Version) -> Self {
        let mut this = self.clone();
        this.version = Some(version);
        this
    }
}

pub mod core_resource_types {
    use super::ResourceType;
    use std::collections::HashMap;

    pub fn git_resource_type() -> ResourceType {
        ResourceType {
            name: String::from("git"),
            type_: String::from("core"),
            source: HashMap::new(),
        }
    }
}

pub fn git_resource(name: &str, uri: &str, branch: &str) -> Resource {
    Resource::new(name, &core_resource_types::git_resource_type())
        .with_source(&vec![("uri", uri), ("branch", branch)])
}
