use crate::resource::Resource;
use crate::resource::TaskImageResource;
use crate::schema::DirPath;
use crate::schema::EnvVars;
use crate::schema::FilePath;
use crate::schema::Identifier;
use crate::step::Step;
use names::Generator;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;

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

fn boolean_is_false(b: &bool) -> bool {
    *b == false
}

#[derive(Debug, Clone, Serialize)]
pub struct Input {
    name: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<DirPath>,
    #[serde(skip_serializing_if = "boolean_is_false")]
    optional: bool,
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
    name: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<DirPath>,
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
    platform: Platform,
    image_resource: TaskImageResource,
    run: Command,
    params: Option<EnvVars>,
    pub(crate) inputs: Option<Vec<Input>>,
    outputs: Option<Vec<Output>>,
}

impl Serialize for TaskConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("TaskConfig", 6)?;
        state.serialize_field("platform", &self.platform)?;
        let anonymouse_resource = self.image_resource.to_anonymouse_resource();
        state.serialize_field("image_resource", &anonymouse_resource)?;
        state.serialize_field("run", &self.run)?;
        if self.params.is_some() {
            state.serialize_field("params", &self.params)?;
        }
        if self.inputs.is_some() {
            state.serialize_field("inputs", &self.inputs)?;
        }
        if self.outputs.is_some() {
            state.serialize_field("inputs", &self.outputs)?;
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

    pub fn with_env(&self, env: &Vec<(&str, &str)>) -> Self {
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
    Resource(Resource),
    Output(Identifier),
}

impl TaskResource {
    pub fn unbound() -> Self {
        Self::Unbound
    }

    pub fn output(identifier: &str) -> Self {
        Self::Output(identifier.to_string())
    }
}

#[derive(Debug, Clone)]
enum TaskKind {
    FromConfig,
    FromFile,
}

#[derive(Debug, Clone)]
pub struct Task {
    kind: TaskKind,
    task: Identifier,
    config: TaskConfig,
    image: Option<TaskImageResource>,
    params: Option<EnvVars>,
    input_mapping: Option<HashMap<String, String>>,
    output_mapping: Option<HashMap<String, String>>,
    // Inputs shouldn't be serialized!!
    inputs: Option<Vec<TaskResource>>,
    // Outputs shouldn't be serialized!!
    outputs: Option<Vec<TaskResource>>,
    on_failure: Option<Box<Step>>,
    on_abort: Option<Box<Step>>,
    on_success: Option<Box<Step>>,
}

impl Serialize for Task {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Task", 6)?;
        state.serialize_field("task", &self.task)?;
        state.serialize_field("config", &self.config)?;
        if self.params.is_some() {
            state.serialize_field("params", &self.params)?;
        }
        if self.input_mapping.is_some() {
            state.serialize_field("input_mapping", &self.input_mapping)?;
        }
        if self.output_mapping.is_some() {
            state.serialize_field("output_mapping", &self.input_mapping)?;
        }
        if self.on_failure.is_some() {
            state.serialize_field("on_failure", &self.on_failure)?;
        }
        if self.on_abort.is_some() {
            state.serialize_field("on_abort", &self.on_abort)?;
        }
        if self.on_success.is_some() {
            state.serialize_field("on_success", &self.on_success)?;
        }

        state.end()
    }
}

impl Task {
    pub fn new() -> Task {
        Self {
            kind: TaskKind::FromConfig,
            task: Generator::default().next().unwrap(),
            config: TaskConfig::linux_default(),
            image: None,
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
        if let TaskKind::FromFile = self.kind {
            panic!(
                ".run() cannot be called in 'task' ('{}') that is initialized from 'file'.",
                self.task.as_str()
            );
        }
        let mut this = self.clone();
        this.config.run = command.clone();
        this
    }

    pub fn with_image(&self) -> Self {
        let this = self.clone();
        this
    }

    pub fn with_image_resource(&self, image_resource: TaskImageResource) -> Self {
        let mut this = self.clone();
        this.config.image_resource = image_resource;
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

    pub fn with_input(&self, input: &TaskResource) -> Self {
        let mut this = self.clone();
        match this.inputs {
            Some(mut inputs) => {
                inputs.push(input.clone());
                this.inputs = Some(inputs);
            }
            None => {
                this.inputs = Some(vec![input.clone()]);
            }
        }
        this
    }

    pub fn with_inputs(&self, inputs: &Vec<&TaskResource>) -> Self {
        let mut this = self.clone();
        this.inputs = Some(
            inputs
                .iter()
                .map(|inp| inp.clone().clone())
                .collect::<Vec<TaskResource>>(),
        );
        this
    }

    pub fn bind_output(&self, output: &str, bind: &mut TaskResource) -> Self {
        let mut this = self.clone();
        match this.outputs {
            Some(mut outputs) => {
                outputs.push(TaskResource::output(output));
                this.outputs = Some(outputs);
            }
            None => this.outputs = Some(vec![TaskResource::output(output)]),
        }
        *bind = TaskResource::output(output);
        this
    }

    pub fn bind_outputs(&self, outputs: &mut Vec<(&str, &mut TaskResource)>) -> Self {
        let mut this = self.clone();
        match this.outputs {
            Some(mut this_outputs) => {
                for o in outputs.iter_mut() {
                    this_outputs.push(TaskResource::output(o.0));
                    *o.1 = TaskResource::output(o.0);
                }
                this.outputs = Some(this_outputs);
            }
            None => {
                let mut this_outputs = vec![];
                for o in outputs.iter_mut() {
                    this_outputs.push(TaskResource::output(o.0));
                    *o.1 = TaskResource::output(o.0);
                }
                this.outputs = Some(this_outputs);
            }
        }
        this
    }

    pub fn outputs(&self) -> &Option<Vec<TaskResource>> {
        &self.outputs
    }

    pub fn inputs(&self) -> &Option<Vec<TaskResource>> {
        &self.inputs
    }

    pub(crate) fn task_config_mut(&mut self) -> &mut TaskConfig {
        &mut self.config
    }

    pub fn mutate_task_config<F: Fn(&TaskConfig) -> TaskConfig>(
        &self,
        task_config_mutator: F,
    ) -> Self {
        let mut this = self.clone();
        let config = this.config;
        this.config = task_config_mutator(&config);
        this
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
