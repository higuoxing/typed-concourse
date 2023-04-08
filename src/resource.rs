use crate::get::Get;
use crate::put::Put;
use crate::schema::Config;
use crate::schema::Identifier;
use crate::schema::Version;
use crate::task::TaskResource;
use git_url_parse::GitUrl;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceTypes {
    DockerImage,
    Git,
    RegistryImage,
    Time,
    Custom {
        name: String,
        type_: Box<ResourceTypes>,
        source: Config,
        params: Config,
    },
}

impl Serialize for ResourceTypes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Resource", 5)?;
        state.serialize_field("name", self.to_string().as_str())?;

        match &self {
            Self::Custom {
                ref type_,
                ref source,
                ref params,
                ..
            } => {
                state.serialize_field("type", type_.to_string().as_str())?;
                if !source.is_empty() {
                    state.serialize_field("source", source)?;
                }
                if !params.is_empty() {
                    state.serialize_field("params", params)?;
                }
            }
            _ => { /* Do nothing. */ }
        }

        state.end()
    }
}

impl ResourceTypes {
    pub fn to_string(&self) -> String {
        match self {
            Self::Git => String::from("git"),
            Self::RegistryImage => String::from("registry-image"),
            Self::DockerImage => String::from("docker-image"),
            Self::Time => String::from("time"),
            Self::Custom { ref name, .. } => name.clone(),
        }
    }

    pub fn new(name: &str, type_: ResourceTypes) -> Self {
        Self::Custom {
            name: name.to_string(),
            type_: Box::new(type_),
            source: BTreeMap::new(),
            params: BTreeMap::new(),
        }
    }

    pub fn with_source(&self, new_source: &[(&str, &str)]) -> Self {
        match self {
            Self::Custom {
                ref name,
                ref type_,
                ref source,
                ref params,
            } => {
                let mut source = source.clone();
                new_source
                    .iter()
                    .map(|(k, v)| source.insert(k.to_string(), v.to_string()))
                    .count();

                Self::Custom {
                    name: name.clone(),
                    type_: type_.clone(),
                    source,
                    params: params.clone(),
                }
            }
            unsupported => panic!(
                "Applying with_source() on resource type '{}' is not allowed",
                unsupported.to_string()
            ),
        }
    }

    pub fn with_params(&self, new_params: &[(&str, &str)]) -> Self {
        match self {
            Self::Custom {
                ref name,
                ref type_,
                ref source,
                ref params,
            } => {
                let mut params = params.clone();
                new_params
                    .iter()
                    .map(|(k, v)| params.insert(k.to_string(), v.to_string()))
                    .count();

                Self::Custom {
                    name: name.clone(),
                    type_: type_.clone(),
                    source: source.clone(),
                    params,
                }
            }
            unsupported => panic!(
                "Applying with_params() on resource type '{}' is not allowed",
                unsupported.to_string()
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub(crate) name: Identifier,
    pub(crate) icon: Option<String>,
    pub(crate) type_: ResourceTypes,
    pub(crate) source: Config,
    pub(crate) trigger: bool,
    pub(crate) version: Option<Version>,
}

impl Serialize for Resource {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Resource", 5)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("type", self.resource_type().to_string().as_str())?;
        if self.icon.is_some() {
            state.serialize_field("icon", &self.icon)?;
        }
        if !self.source.is_empty() {
            state.serialize_field("source", &self.source)?;
        }
        if self.version.is_some() {
            state.serialize_field("version", &self.version)?;
        }
        state.end()
    }
}

impl Resource {
    pub fn new(name: &str, res_type: &ResourceTypes) -> Self {
        Self {
            name: name.to_string(),
            icon: None,
            type_: res_type.clone(),
            source: BTreeMap::new(),
            trigger: false,
            version: None,
        }
    }

    pub(crate) fn trigger(&self) -> bool {
        self.trigger
    }

    pub fn git(uri: &str, branch: &str) -> Self {
        let git_url = GitUrl::parse(uri)
            .expect(format!("The URI of given git resource '{}' is not valid", uri).as_str());

        let mut source = BTreeMap::new();
        source.insert(String::from("uri"), uri.to_string());
        if branch != "" {
            source.insert(String::from("branch"), branch.to_string());
        }

        let name = if branch == "" {
            git_url.name
        } else {
            format!("{}.{}", git_url.name, branch)
        };

        Self {
            name,
            type_: ResourceTypes::Git,
            icon: if uri.contains("github") {
                Some(String::from("github"))
            } else if uri.contains("gitlab") {
                Some(String::from("gitlab"))
            } else {
                None
            },
            source,
            trigger: false,
            version: None,
        }
    }

    pub fn time(interval: &str) -> Self {
        Self {
            name: format!("every-{}", interval),
            type_: ResourceTypes::Time,
            icon: Some(String::from("clock-outline")),
            source: [("interval", interval)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            trigger: false,
            version: None,
        }
    }

    pub fn registry_image(repository: &str) -> Self {
        Self {
            name: repository.to_string(),
            type_: ResourceTypes::RegistryImage,
            icon: Some(String::from("docker")),
            source: [("repository", repository)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            trigger: false,
            version: None,
        }
    }

    pub fn docker_image() -> Self {
        todo!()
    }

    pub fn with_name(&self, name: &str) -> Self {
        let mut this = self.clone();
        this.name = name.to_string();
        this
    }

    pub fn with_icon(&self, icon: &str) -> Self {
        let mut this = self.clone();

        this.icon = if icon == "" {
            None
        } else {
            Some(icon.to_string())
        };

        this
    }

    pub fn with_trigger(&self, trigger: bool) -> Self {
        let mut this = self.clone();
        this.trigger = trigger;
        this
    }

    pub fn name(&self) -> Identifier {
        self.name.clone()
    }

    pub fn resource_type(&self) -> &ResourceTypes {
        &self.type_
    }

    pub fn with_source(mut self, source: &[(&str, &str)]) -> Self {
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

    pub fn as_put_resource(&self) -> Put {
        Put::from("", self)
    }

    pub fn as_task_input_resource(&self) -> TaskResource {
        TaskResource::Resource {
            resource: self.clone(),
            get_as: None,
            map_to: None,
        }
    }

    pub fn as_task_image_resource(&self) -> TaskImageResource {
        TaskImageResource {
            resource: self.clone(),
        }
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
    pub fn from(type_: &str, source: &[(&str, &str)]) -> Self {
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

    pub fn with_params(&self, params: &[(&str, &str)]) -> Self {
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskImageResourceType {
    RegistryImage,
    DockerImage,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct TaskImageResource {
    pub(crate) resource: Resource,
}

impl TaskImageResource {
    pub(crate) fn to_anonymouse_resource(&self) -> AnonymousResource {
        AnonymousResource {
            type_: self.resource.type_.to_string(),
            source: self.resource.source.clone(),
            params: None,
            version: self.resource.version.clone(),
        }
    }
}
