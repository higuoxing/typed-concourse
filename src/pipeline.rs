use crate::errors::Errors;
use crate::job::Job;
use crate::resource::Resource;
use crate::resource::ResourceType;
use crate::schema::Identifier;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Pipeline {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    jobs: Vec<Job>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    resources: HashMap<Identifier, Resource>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    resource_types: HashMap<Identifier, ResourceType>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            jobs: vec![],
            resources: HashMap::new(),
            resource_types: HashMap::new(),
        }
    }

    pub fn append(mut self, job: Job) -> Result<Self, Errors> {
        self.jobs.push(job);
        Ok(self)
    }

    pub fn jobs(&self) -> &Vec<Job> {
        &self.jobs
    }
}
