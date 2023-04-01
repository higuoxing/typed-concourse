use crate::errors::Errors;
use crate::resource::AnonymousResource;
use crate::schema::FilePath;
use crate::schema::Identifier;
use crate::schema::Version;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Darwin,
    Windows,
}

#[derive(Serialize)]
pub struct Command {
    path: FilePath,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
}

#[derive(Serialize)]
pub struct TaskConfig {
    platform: Platform,
    image_resource: AnonymousResource,
    run: Command,
}

#[derive(Serialize)]
pub enum GetVersion {
    Latest,
    Every,
    Version(Version),
}

#[derive(Serialize)]
pub struct GetStep {
    get: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<GetVersion>,
}

#[derive(Serialize)]
pub struct PutStep {
    put: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<Identifier>,
}

#[derive(Serialize)]
pub struct TaskStep {
    task: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<TaskConfig>,
}

#[derive(Serialize)]
pub enum Step {
    Get(GetStep),
    Put(PutStep),
    Task(TaskStep),
}

impl Step {
    pub fn get(res_name: &str) -> Step {
        Step::Get(GetStep {
            get: res_name.to_string(),
            resource: None,
            version: None,
        })
    }

    pub fn relabel(self, new_name: &str) -> Result<Step, Errors> {
        match self {
            Self::Get(get_step) => match get_step.resource {
                Some(res_name) => Err(Errors::from(format!(
                        "Resource cannot be relabeled twice!\nThe resource '{}' has already been relabeled with name '{}'.",
                        get_step.get, res_name,
                    )
                    .as_str(),
                )),
                None => Ok(Self::Get(GetStep {
                    get: new_name.to_string(),
                    resource: Some(get_step.get),
                    version: get_step.version,
                }))
            },
            Self::Put(_) => Err(Errors::from(
                "Cannot relabel the resource name in the 'put' step.",
            )),
            Self::Task(_) => Err(Errors::from(
                "Cannot relabel the resource name in the 'task' step.",
            )),
        }
    }

    pub fn put(res_name: &str) -> Step {
        Step::Put(PutStep {
            put: res_name.to_string(),
            resource: None,
        })
    }
}
