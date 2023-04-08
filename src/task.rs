use crate::resource::Resource;
use crate::resource::TaskImageResource;
use crate::schema::DirPath;
use crate::schema::EnvVars;
use crate::schema::FilePath;
use crate::schema::Identifier;
use crate::schema::Vars;
use crate::step::Step;
use names::Generator;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, Serialize)]
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
    pub fn new(path: &str, args: &[&str]) -> Self {
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

fn boolean_is_false(b: &bool) -> bool {
    *b == false
}

#[derive(Debug, Clone, Serialize)]
pub struct Input {
    pub(crate) name: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<DirPath>,
    #[serde(skip_serializing_if = "boolean_is_false")]
    pub(crate) optional: bool,
}

impl Input {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            path: None,
            optional: false,
        }
    }

    pub fn with_path(&self, path: &str) -> Self {
        let mut this = self.clone();
        this.path = Some(path.to_string());
        this
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Output {
    pub(crate) name: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<DirPath>,
}

impl Output {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            path: None,
        }
    }

    pub fn with_path(&self, path: &str) -> Self {
        let mut this = self.clone();
        this.path = Some(path.to_string());
        this
    }
}

#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub(crate) platform: Platform,
    pub(crate) image_resource: TaskImageResource,
    pub(crate) run: Command,
    pub(crate) params: Option<EnvVars>,
    pub(crate) inputs: Option<Vec<Input>>,
    pub(crate) outputs: Option<Vec<Output>>,
    // When both the 'image' tag in 'task' struct and 'image_resource' tag in
    // the 'task_config' struct exist, we don't serialize the 'image_resource' tag,
    // since the 'image' tag has higher priority.
    pub(crate) serialize_image_resource: bool,
}

impl Serialize for TaskConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("TaskConfig", 6)?;
        state.serialize_field("platform", &self.platform)?;
        if self.serialize_image_resource {
            let anonymouse_resource = self.image_resource.to_anonymouse_resource();
            state.serialize_field("image_resource", &anonymouse_resource)?;
        }
        state.serialize_field("run", &self.run)?;
        if self.params.is_some() {
            state.serialize_field("params", &self.params)?;
        }
        if self.inputs.is_some() {
            state.serialize_field("inputs", &self.inputs)?;
        }
        if self.outputs.is_some() {
            state.serialize_field("outputs", &self.outputs)?;
        }
        state.end()
    }
}

impl TaskConfig {
    pub fn linux_default() -> Self {
        Self {
            platform: Platform::Linux,
            image_resource: Resource::registry_image("busybox").as_task_image_resource(),
            run: Command::new("echo", &vec!["hello, world!"]),
            params: None,
            inputs: None,
            outputs: None,
            serialize_image_resource: true,
        }
    }

    pub fn windows_default() -> Self {
        todo!("Generating default config for Windows platform")
    }

    pub fn darwin_default() -> Self {
        todo!("Generating default config for Darwin platform")
    }

    pub fn with_platform(&self, platform: Platform) -> Self {
        let mut this = self.clone();
        this.platform = platform;
        this
    }

    pub fn with_image_resource(&self, image_resource: &TaskImageResource) -> Self {
        let mut this = self.clone();
        this.image_resource = image_resource.clone();
        this
    }

    pub fn run(&self, command: &Command) -> Self {
        let mut this = self.clone();
        this.run = command.clone();
        this
    }

    pub fn with_env(&self, env: &[(&str, &str)]) -> Self {
        let mut this = self.clone();
        this.params = Some(
            env.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        );
        this
    }

    pub fn with_inputs(&self, inputs: Vec<Input>) -> Self {
        let mut this = self.clone();
        this.inputs = Some(inputs);
        this
    }

    pub fn with_outputs(&self, outputs: Vec<Output>) -> Self {
        let mut this = self.clone();
        this.outputs = Some(outputs);
        this
    }
}

#[derive(Debug, Clone)]
pub enum TaskResource {
    Unbound,
    Resource {
        resource: Resource,
        get_as: Option<String>,
        map_to: Option<String>,
    },
    Output {
        name: Identifier,
        map_to: Option<String>,
        map_from: Option<String>,
    },
}

impl TaskResource {
    pub fn unbound() -> Self {
        Self::Unbound
    }

    pub fn map_from(&self, map_from_name: &str) -> Self {
        match self {
            Self::Output {
                ref name,
                ref map_to,
                ..
            } => Self::Output {
                name: name.clone(),
                map_from: Some(map_from_name.to_string()),
                map_to: map_to.clone(),
            },
            _ => {
                panic!("map_from() can only be used with 'Output' TaskResource")
            }
        }
    }

    pub fn get_as(&self, name: &str) -> TaskResource {
        match &self {
            Self::Resource {
                ref resource,
                ref map_to,
                ..
            } => Self::Resource {
                resource: resource.clone(),
                get_as: Some(name.to_string()),
                map_to: map_to.clone(),
            },
            _ => {
                panic!(
                    "get_as() can only apply on TaskResource created from as_task_input_resource()"
                )
            }
        }
    }

    pub fn map_to(&self, name: &str) -> TaskResource {
        match &self {
            Self::Resource {
                ref resource,
                ref get_as,
                ..
            } => Self::Resource {
                resource: resource.clone(),
                get_as: get_as.clone(),
                map_to: Some(name.to_string()),
            },
            Self::Output {
                ref name,
                ref map_from,
                ..
            } => Self::Output {
                name: name.clone(),
                map_to: Some(name.to_string()),
                map_from: map_from.clone(),
            },
            _ => panic!("map_to() cannot apply on unbound TaskResource"),
        }
    }

    pub fn bind(&self, to: &mut Self) -> Self {
        *to = self.clone();
        self.clone()
    }

    pub fn output(identifier: &str) -> Self {
        Self::Output {
            name: identifier.to_string(),
            map_to: None,
            map_from: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TaskDef {
    File { file: FilePath, vars: Vars },
    Config { config: TaskConfig },
}

impl TaskDef {
    pub(crate) fn is_from_file(&self) -> bool {
        match self {
            Self::File { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    task: Identifier,
    pub(crate) task_def: TaskDef,
    pub(crate) image: Option<TaskImageResource>,
    priviledged: bool,
    // TODO: container-limit.
    params: Option<EnvVars>,
    pub(crate) input_mapping: Option<BTreeMap<String, String>>,
    pub(crate) output_mapping: Option<BTreeMap<String, String>>,

    // Inputs shouldn't be serialized!!
    pub(crate) inputs: Option<Vec<TaskResource>>,
    // Outputs shouldn't be serialized!!
    pub(crate) outputs: Option<Vec<TaskResource>>,
    // Hooks.
    on_failure: Option<Box<Step>>,
    on_abort: Option<Box<Step>>,
    on_success: Option<Box<Step>>,
}

impl Serialize for Task {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Task", 9)?;

        state.serialize_field("task", &self.task)?;

        match self.task_def {
            TaskDef::Config { ref config, .. } => {
                let mut config = config.clone();
                config.serialize_image_resource = self.image.is_none();
                state.serialize_field("config", &config)?;
            }
            TaskDef::File { ref file, .. } => {
                state.serialize_field("file", file)?;
            }
        }

        if let Some(ref image) = self.image.as_ref() {
            state.serialize_field("image", image.resource.name.as_str())?;
        }

        if self.priviledged {
            state.serialize_field("priviledged", &true)?;
        }

        if let Some(ref input_mapping) = self.input_mapping.as_ref() {
            state.serialize_field("input_mapping", input_mapping)?;
        }

        if let Some(ref output_mapping) = self.output_mapping.as_ref() {
            state.serialize_field("output_mapping", output_mapping)?;
        }

        if let Some(ref on_failure) = self.on_failure.as_ref() {
            state.serialize_field("on_failure", on_failure.as_ref())?;
        }

        if let Some(ref on_abort) = self.on_abort.as_ref() {
            state.serialize_field("on_abort", on_abort.as_ref())?;
        }

        if let Some(ref on_success) = self.on_success.as_ref() {
            state.serialize_field("on_success", on_success.as_ref())?;
        }

        state.end()
    }
}

impl Task {
    pub fn new() -> Task {
        Self {
            task: Generator::default().next().unwrap(),
            task_def: TaskDef::Config {
                config: TaskConfig::linux_default(),
            },
            image: None,
            priviledged: false,
            params: None,
            input_mapping: None,
            output_mapping: None,
            inputs: None,
            outputs: None,
            on_abort: None,
            on_failure: None,
            on_success: None,
        }
    }

    pub fn from_file(file: &str) -> Task {
        Self {
            task: Generator::default().next().unwrap(),
            task_def: TaskDef::File {
                file: file.to_string(),
                vars: BTreeMap::new(),
            },
            image: None,
            priviledged: false,
            params: None,
            input_mapping: None,
            output_mapping: None,
            inputs: None,
            outputs: None,
            on_abort: None,
            on_failure: None,
            on_success: None,
        }
    }

    pub fn with_platform(&self, platform: Platform) -> Self {
        let mut this = self.clone();
        this = this.mutate_task_config(|task_config| task_config.with_platform(platform));
        this
    }

    pub fn with_name(&self, name: &str) -> Self {
        let mut this = self.clone();
        this.task = name.to_string();
        this
    }

    pub fn run(&self, command: &Command) -> Self {
        match self.task_def {
            TaskDef::File { .. } => panic!(
                ".run() cannot be called in 'task' ('{}') that is initialized from 'file'.",
                self.task.as_str()
            ),
            TaskDef::Config { ref config } => {
                let mut this = self.clone();
                let mut this_config = config.clone();
                this_config.run = command.clone();
                this.task_def = TaskDef::Config {
                    config: this_config,
                };
                this
            }
        }
    }

    pub fn with_image(&self, image: TaskImageResource) -> Self {
        let mut this = self.clone();
        this.image = Some(image);
        this
    }

    pub fn with_image_resource(&self, image_resource: TaskImageResource) -> Self {
        match self.task_def {
            TaskDef::File { .. } => panic!(".with_image_resource() cannot be called in 'task' ('{}') that is initialized from 'file'.", self.task.as_str()),
            TaskDef::Config {
                ref config,
            } => {
                let mut this_config = config.clone();
                this_config.image_resource = image_resource;
                let mut this = self.clone();
                this.task_def = TaskDef::Config {
                    config: this_config,
                };
                this
            }
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

    pub fn with_inputs(&self, inputs: &[&TaskResource]) -> Self {
        let mut this = self.clone();

        this.inputs = Some(
            inputs
                .iter()
                .map(|inp| inp.clone().clone())
                .collect::<Vec<TaskResource>>(),
        );

        let input_mapping = if !self.task_def.is_from_file() {
            inputs
                .iter()
                .filter_map(|inp| match *inp {
                    TaskResource::Resource {
                        ref resource,
                        ref map_to,
                        ref get_as,
                    } => {
                        if let Some(ref map_to_name) = map_to {
                            if let Some(ref get_as_name) = get_as {
                                Some((map_to_name.clone(), get_as_name.clone()))
                            } else {
                                Some((map_to_name.clone(), resource.name.clone()))
                            }
                        } else {
                            None
                        }
                    }
                    TaskResource::Output {
                        ref name,
                        ref map_to,
                        ..
                    } => {
                        if let Some(ref map_to_name) = map_to {
                            Some((map_to_name.clone(), name.clone()))
                        } else {
                            None
                        }
                    }
                    _ => {
                        panic!("Cannot pass unbound TaskResource to with_inputs()");
                    }
                })
                .collect::<BTreeMap<String, String>>()
        } else {
            inputs
                .iter()
                .filter_map(|ref inp| match *inp {
                    TaskResource::Resource {
                        ref resource,
                        ref get_as,
                        ref map_to,
                    } => {
                        if let Some(ref map_to_name) = map_to {
                            if let Some(ref get_as_name) = get_as {
                                Some((map_to_name.clone(), get_as_name.clone()))
                            } else {
                                Some((map_to_name.clone(), resource.name().clone()))
                            }
                        } else {
                            None
                        }
                    }
                    TaskResource::Output {
                        ref name,
                        ref map_to,
                        ..
                    } => {
                        if let Some(ref map_to_name) = map_to {
                            Some((map_to_name.clone(), name.clone()))
                        } else {
                            None
                        }
                    }
                    TaskResource::Unbound { .. } => {
                        panic!("Cannot pass unbound TaskResource to with_inputs()");
                    }
                })
                .collect::<BTreeMap<String, String>>()
        };

        if !input_mapping.is_empty() {
            this.input_mapping = Some(input_mapping);
        }

        this
    }

    pub fn with_outputs(&self, outputs: &[&TaskResource]) -> Self {
        if outputs.is_empty() {
            self.clone()
        } else {
            let mut this = self.clone();
            let mut output_mapping = BTreeMap::new();
            let outputs = outputs
                .iter()
                .map(|v| match **v {
                    TaskResource::Output {
                        ref name,
                        ref map_from,
                        ..
                    } => {
                        if let Some(ref map_from_name) = map_from {
                            output_mapping.insert(map_from_name.clone(), name.clone());
                        }
                        v.clone().clone()
                    }
                    _ => panic!("Only 'Output' TaskResource can be used in with_outputs()"),
                })
                .collect::<Vec<TaskResource>>();
            this.outputs = Some(outputs);
            this.output_mapping = if !output_mapping.is_empty() {
                Some(output_mapping)
            } else {
                None
            };
            this
        }
    }

    pub fn mutate_task_config<F: Fn(&TaskConfig) -> TaskConfig>(
        &self,
        task_config_mutator: F,
    ) -> Self {
        match self.task_def {
            TaskDef::File { .. } => panic!(".mutate_task_config() cannot be called in 'task' ('{}') that is initialized from 'file'.", self.task.as_str()),
            TaskDef::Config {
                ref config,
            } => {
                let mut this = self.clone();
                this.task_def = TaskDef::Config {
                    config: task_config_mutator(config),
                };
                this
            }
        }
    }

    pub fn to_step(&self) -> Step {
        Step::Task(self.clone())
    }

    pub fn on_failure(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.on_failure = Some(Box::new(step));
        this
    }

    pub fn on_abort(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.on_abort = Some(Box::new(step));
        this
    }

    pub fn on_success(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.on_success = Some(Box::new(step));
        this
    }
}
