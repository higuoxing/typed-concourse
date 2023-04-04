use crate::get::Get;
use crate::schema::Config;
use crate::schema::Identifier;
use crate::schema::Version;
use crate::task::TaskResource;
use git_url_parse::GitUrl;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceTypes {
    DockerImage,
    Git,
    RegistryImage,
    Time,
    Custom(String),
}

impl ResourceTypes {
    pub fn to_string(&self) -> String {
        match self {
            Self::Git => String::from("git"),
            Self::RegistryImage => String::from("registry-image"),
            Self::DockerImage => String::from("docker-image"),
            Self::Time => String::from("time"),
            Self::Custom(ref custom) => custom.clone(),
        }
    }

    pub fn from(resource_type: &str) -> Self {
        match resource_type {
            "git" => Self::Git,
            "registry-image" => Self::RegistryImage,
            "docker-image" => Self::DockerImage,
            "time" => Self::Time,
            custom => Self::Custom(custom.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceType {
    name: Identifier,
    #[serde(rename(serialize = "type"))]
    type_: ResourceTypes,
    source: Config,
}

impl ResourceType {
    pub fn name(&self) -> Identifier {
        self.name.clone()
    }

    pub fn type_(&self) -> ResourceTypes {
        self.type_.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Resource {
    name: Identifier,
    icon: Option<String>,
    type_: ResourceTypes,
    source: Config,
    trigger: bool,
}

impl Serialize for Resource {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Resource", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("type", &self.type_)?;
        if self.icon.is_some() {
            state.serialize_field("icon", &self.icon)?;
        }
        if !self.source.is_empty() {
            state.serialize_field("source", &self.source)?;
        }
        state.end()
    }
}

impl Resource {
    pub fn new(name: &str, res_type: ResourceTypes) -> Self {
        Self {
            name: name.to_string(),
            icon: None,
            type_: res_type,
            source: HashMap::new(),
            trigger: false,
        }
    }

    pub(crate) fn trigger(&self) -> bool {
        self.trigger
    }

    pub fn git(uri: &str, branch: &str) -> Self {
        let git_url = GitUrl::parse(uri)
            .expect(format!("The URI of given git resource '{}' is not valid", uri).as_str());

        let mut source = HashMap::new();
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
        }
    }

    pub fn registry_image() -> Self {
        todo!()
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

    pub fn as_task_input(&self) -> TaskResource {
        TaskResource::Resource(self.clone())
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskImageResourceType {
    RegistryImage,
    DockerImage,
    Custom(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskImageResource {
    #[serde(rename(serialize = "type"))]
    type_: TaskImageResourceType,
    source: Config,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Config>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<Version>,
}

impl TaskImageResource {
    pub fn registry_image(repository: &str) -> Self {
        let mut source = HashMap::new();
        source.insert(String::from("repository"), repository.to_string());
        Self {
            type_: TaskImageResourceType::RegistryImage,
            source,
            params: None,
            version: None,
        }
    }

    pub fn docker_image(repository: &str) -> Self {
        let mut source = HashMap::new();
        source.insert(String::from("repository"), repository.to_string());
        Self {
            type_: TaskImageResourceType::DockerImage,
            source,
            params: None,
            version: None,
        }
    }

    pub fn with_tag(&self, tag: &str) -> Self {
        let mut this = self.clone();
        this.source.insert(String::from("tag"), tag.to_string());
        this
    }

    pub fn with_source(&self, source: &Vec<(&str, &str)>) -> Self {
        let mut this = self.clone();
        source
            .iter()
            .map(|(k, v)| this.source.insert(k.to_string(), v.to_string()))
            .count();
        this
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
