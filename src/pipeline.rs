use crate::job::Job;
use crate::resource::Resource;
use crate::resource::ResourceTypes;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Pipeline {
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
            jobs: vec![],
            resources: vec![],
            resource_types: vec![],
        }
    }

    pub fn with_resources(&self, resources: Vec<Resource>) -> Self {
        let mut this = self.clone();
        this.resources = resources;
        this
    }

    pub fn with_resource_types(&self, resource_types: Vec<ResourceTypes>) -> Self {
        let mut this = self.clone();
        this.resource_types = resource_types;
        this
    }

    pub fn append(&self, job: Job) -> Self {
        let mut this = self.clone();
        this.jobs.push(job);
        this
    }

    pub fn jobs(&self) -> &Vec<Job> {
        &self.jobs
    }
}
