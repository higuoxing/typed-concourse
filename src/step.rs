use crate::core::FilePath;
use crate::core::Identifier;
use crate::core::Number;
use crate::core::Version;
use crate::errors::Errors;
use crate::resource::AnonymousResource;
use crate::resource::Resource;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Darwin,
    Windows,
}

#[derive(Debug, Clone, Serialize)]
pub struct Command {
    path: FilePath,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskConfig {
    platform: Platform,
    image_resource: AnonymousResource,
    run: Command,
}

#[derive(Debug, Clone)]
pub struct GetStep {
    get: Identifier,
    resource: Resource,
    version: Option<Version>,
    trigger: bool,
}

impl Serialize for GetStep {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("GetStep", 4)?;
        if !self.get.is_empty() {
            state.serialize_field("get", &self.get)?;
            state.serialize_field("resource", &self.resource.name())?;
        } else {
            state.serialize_field("get", &self.resource.name())?;
        }

        match self.version.as_ref() {
            Some(version) => state.serialize_field("version", version)?,
            None => { /* Do nothing. */ }
        }

        if self.trigger {
            state.serialize_field("trigger", &self.trigger)?;
        }

        state.end()
    }
}

impl GetStep {
    pub fn from(identifier: &str, resource: &Resource, version: Option<Version>) -> Self {
        Self {
            get: identifier.to_string(),
            resource: resource.clone(),
            version,
            trigger: false,
        }
    }

    pub fn with_version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    pub fn trigger_new_build(mut self) -> Self {
        self.trigger = true;
        self
    }

    pub fn resource(&self) -> &Resource {
        &self.resource
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PutStep {
    put: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<Identifier>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskStep {
    task: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<TaskConfig>,
}

#[derive(Debug, Clone)]
pub enum InParallelStep {
    Steps(Vec<Step>),
    InParallelConfig {
        steps: Vec<Step>,
        limit: Option<Number>,
        fail_fast: bool,
    },
}

impl Serialize for InParallelStep {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            InParallelStep::Steps(ref steps) => {
                let mut state = serializer.serialize_struct("InParallel", 1)?;
                state.serialize_field("in_parallel", steps)?;
                state.end()
            }
            InParallelStep::InParallelConfig { .. } => todo!(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Step {
    Get(GetStep),
    Put(PutStep),
    Task(TaskStep),
    InParallel(InParallelStep),
}

impl Step {
    pub fn rename(self, new_name: &str) -> Result<Self, Errors> {
        match self {
            Self::Get(mut get_step) => {
                if !get_step.get.is_empty() {
                    Err(Errors::from(
                        "Cannot rename a resource in the 'get' step more than twice.",
                    ))
                } else {
                    get_step.get = new_name.to_string();
                    Ok(Step::Get(get_step))
                }
            }
            _ => Err(Errors::from(
                "Rename the resource can only be used in the 'get' step.",
            )),
        }
    }

    pub fn with_version(self, version: Version) -> Result<Self, Errors> {
        match self {
            Self::Get(mut get_step) => {
                get_step = get_step.with_version(version);
                Ok(Step::Get(get_step))
            }
            _ => Err(Errors::from(
                "Setting resource version can only be used in the 'get' step.",
            )),
        }
    }

    pub fn trigger_new_build(self) -> Result<Self, Errors> {
        match self {
            Self::Get(mut get_step) => {
                get_step = get_step.trigger_new_build();
                Ok(Step::Get(get_step))
            }
            _ => Err(Errors::from(
                "trigger_new_build() can only be used in the 'get' step.",
            )),
        }
    }
}
