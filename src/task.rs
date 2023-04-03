use crate::resource::TaskImageResource;
use crate::schema::EnvVars;
use crate::schema::FilePath;
use crate::schema::Identifier;
use crate::step::Step;
use names::Generator;
use serde::Serialize;

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
    image_resource: TaskImageResource,
    run: Command,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<EnvVars>,
    inputs: Option<Vec<Identifier>>,
}

impl TaskConfig {
    pub fn linux_default() -> Self {
        Self {
            platform: Platform::Linux,
            image_resource: TaskImageResource::registry_image("busybox"),
            run: Command::new("echo", &vec!["hello, world!"]),
            params: None,
            inputs: None,
        }
    }

    pub fn windows_default() -> Self {
        todo!("Generating default config for Windows platform")
    }

    pub fn darwin_default() -> Self {
        todo!("Generating default config for Darwin platform")
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

    pub fn to_step(&self) -> Step {
        Step::Task(self.clone())
    }
}
