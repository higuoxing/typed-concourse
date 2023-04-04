use crate::resource::Resource;
use crate::resource::TaskImageResource;
use crate::schema::EnvVars;
use crate::schema::FilePath;
use crate::schema::Identifier;
use crate::step::Step;
use names::Generator;
use serde::Serialize;
use std::collections::HashMap;

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

fn boolean_is_false(b: &bool) -> bool {
    *b == false
}

#[derive(Debug, Clone, Serialize)]
pub struct Input {
    name: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "boolean_is_false")]
    optional: bool,
}

impl Input {
    pub fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            path: None,
            optional: false,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) inputs: Option<Vec<Input>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outputs: Option<Vec<Identifier>>,
}

impl TaskConfig {
    pub fn linux_default() -> Self {
        Self {
            platform: Platform::Linux,
            image_resource: TaskImageResource::registry_image("busybox"),
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

    pub(crate) fn inputs_mut(&mut self) -> Option<&mut Vec<Input>> {
        self.inputs.as_mut()
    }
}

#[derive(Debug, Clone)]
pub enum TaskResource {
    Uninitialized,
    Resource(Resource),
    Output(Identifier),
}

impl TaskResource {
    pub fn default() -> Self {
        Self::Uninitialized
    }

    pub fn output(identifier: &str) -> Self {
        Self::Output(identifier.to_string())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    input_mapping: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_mapping: Option<HashMap<String, String>>,
    #[serde(skip_serializing)]
    inputs: Option<Vec<TaskResource>>,
    #[serde(skip_serializing)]
    outputs: Option<Vec<TaskResource>>,
}

impl Task {
    pub fn from_file(identifier: &str, file: &str) -> Self {
        Self {
            task: identifier.to_string(),
            config: None,
            file: Some(file.to_string()),
            image: None,
            params: None,
            input_mapping: None,
            output_mapping: None,
            inputs: None,
            outputs: None,
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
            input_mapping: None,
            output_mapping: None,
            inputs: None,
            outputs: None,
        }
    }

    pub fn with_name(&self, name: &str) -> Self {
        let mut this = self.clone();
        this.task = name.to_string();
        this
    }

    pub fn with_image_resource(&self, image_resource: TaskImageResource) -> Self {
        let mut this = self.clone();
        match this.config {
            Some(mut config) => {
                config.image_resource = image_resource;
                this.config = Some(config);
                this
            }
            None => {
                panic!(
                    "with_image_resource() cannot be used with 'task's instantiated from 'file'."
                )
            }
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

    pub(crate) fn task_config_mut(&mut self) -> Option<&mut TaskConfig> {
        self.config.as_mut()
    }

    pub fn mutate_task_config<F: Fn(&TaskConfig) -> TaskConfig>(
        &self,
        task_config_mutator: F,
    ) -> Self {
        let mut this = self.clone();
        let config = this.config.expect(
            format!(
                "Task '{}' is not initialized from TaskConfig",
                self.task.as_str()
            )
            .as_str(),
        );

        this.config = Some(task_config_mutator(&config));
        this
    }

    pub fn to_step(&self) -> Step {
        Step::Task(self.clone())
    }
}
