use crate::errors::Errors;
use crate::job::Job;
use crate::resource::Resource;
use crate::resource::ResourceType;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Pipeline {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    jobs: Vec<Job>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    resource_types: Vec<ResourceType>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            jobs: vec![],
            resources: vec![],
            resource_types: vec![],
        }
    }

    pub fn with_resources(mut self, resources: Vec<Resource>) -> Result<Self, Errors> {
        self.resources = resources;
        Ok(self)
    }

    pub fn with_resource_types(
        mut self,
        resource_types: Vec<ResourceType>,
    ) -> Result<Self, Errors> {
        self.resource_types = resource_types;
        Ok(self)
    }

    pub fn append(mut self, job: Job) -> Result<Self, Errors> {
        self.jobs.push(job);
        Ok(self)
    }

    pub fn jobs(&self) -> &Vec<Job> {
        &self.jobs
    }
}
