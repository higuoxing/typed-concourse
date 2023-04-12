use crate::job::Job;
use crate::resource::Resource;
use crate::resource::ResourceTypes;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DisplayConfig {
    pub(crate) background_image: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pipeline {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) display: Option<DisplayConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) jobs: Vec<Job>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) resource_types: Vec<ResourceTypes>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            display: None,
            jobs: vec![],
            resources: vec![],
            resource_types: vec![],
        }
    }

    pub fn with_background(mut self, uri: &str) -> Self {
        self.display = Some(DisplayConfig {
            background_image: uri.to_string(),
        });
        self
    }

    pub fn with_resources(mut self, resources: Vec<Resource>) -> Self {
        self.resources = resources;
        self
    }

    pub fn with_resource_types(mut self, resource_types: Vec<ResourceTypes>) -> Self {
        self.resource_types = resource_types;
        self
    }

    pub fn append(mut self, job: Job) -> Self {
        self.jobs.push(job);
        self
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }
}
