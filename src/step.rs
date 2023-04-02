use crate::core::EnvVars;
use crate::core::FilePath;
use crate::core::Identifier;
use crate::core::Number;
use crate::core::Version;
use crate::errors::Errors;
use crate::resource::AnonymousResource;
use crate::resource::Resource;
use names::Generator;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;

#[derive(Debug, Clone)]
pub struct Get {
    get: Identifier,
    resource: Resource,
    version: Option<Version>,
    trigger: bool,
}

impl Serialize for Get {
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

impl Get {
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

    pub fn get(&self) -> Step {
        Step::Get(self.clone())
    }

    pub fn get_as(&self, alias: &str) -> Step {
        let mut this = self.clone();
        this.get = alias.to_string();
        Step::Get(this)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Put {
    put: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<Identifier>,
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,
}

impl Command {
    pub fn new(path: &str, args: &Vec<&str>) -> Self {
        Self {
            path: path.to_string(),
            args: if args.is_empty() {
                None
            } else {
                Some(args.iter().map(|s| s.to_string()).collect())
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskConfig {
    platform: Platform,
    image_resource: AnonymousResource,
    run: Command,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<EnvVars>,
    inputs: Option<Vec<Identifier>>,
}

impl TaskConfig {
    pub fn linux_default() -> Self {
        Self {
            platform: Platform::Linux,
            image_resource: AnonymousResource::from(
                "registry-image",
                &vec![("repository", "busybox")],
            ),
            run: Command::new("echo", &vec!["hello, world!"]),
            params: None,
            inputs: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    task: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<TaskConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<FilePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<EnvVars>,
}

impl Task {
    pub fn from_file(identifier: &str, file: &str) -> Self {
        Self {
            task: identifier.to_string(),
            config: None,
            file: Some(file.to_string()),
            image: None,
            params: None,
        }
    }

    pub fn linux() -> Task {
        let task_config = TaskConfig::linux_default();
        Self {
            task: Generator::default().next().unwrap(),
            config: Some(task_config),
            file: None,
            image: None,
            params: None,
        }
    }

    pub fn with_name(&self, name: &str) -> Self {
        let mut this = self.clone();
        this.task = name.to_string();
        this
    }

    pub fn with_inputs(&self, inputs: &Vec<Resource>) -> Result<Self, Errors> {
        let mut this = self.clone();
        match this.config {
            Some(mut config) => {
                todo!()
            }
            None => Err(Errors::from(
                "Cannot specify inputs for a 'task' that is instantiated from 'file'",
            )),
        }
    }

    pub fn to_step(&self) -> Step {
        Step::Task(self.clone())
    }
}

#[derive(Debug, Clone)]
pub enum InParallel {
    Steps(Vec<Step>),
    InParallelConfig {
        steps: Vec<Step>,
        limit: Option<Number>,
        fail_fast: bool,
    },
}

impl Serialize for InParallel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            InParallel::Steps(ref steps) => {
                let mut state = serializer.serialize_struct("InParallel", 1)?;
                state.serialize_field("in_parallel", steps)?;
                state.end()
            }
            InParallel::InParallelConfig { .. } => todo!(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Step {
    Get(Get),
    Put(Put),
    Task(Task),
    InParallel(InParallel),
}
