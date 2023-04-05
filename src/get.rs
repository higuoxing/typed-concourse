use crate::job::Job;
use crate::resource::Resource;
use crate::schema::Identifier;
use crate::schema::Version;
use crate::step::Step;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;

#[derive(Debug, Clone)]
pub struct Get {
    pub(crate) get: Identifier,
    pub(crate) resource: Resource,
    pub(crate) version: Option<Version>,
    pub(crate) trigger: bool,
    pub(crate) passed: Option<Vec<Job>>,
}

impl Serialize for Get {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("GetStep", 5)?;
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

        match self.passed.as_ref() {
            Some(passed) => state.serialize_field(
                "passed",
                &passed
                    .iter()
                    .map(|job| job.name())
                    .collect::<Vec<Identifier>>(),
            )?,
            None => { /* Do nothing. */ }
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
            trigger: resource.trigger(),
            passed: None,
        }
    }

    pub fn with_version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_trigger(&self, trigger: bool) -> Self {
        let mut this = self.clone();
        this.trigger = trigger;
        this
    }

    pub fn with_passed(&self, jobs: &[Job]) -> Self {
        let mut this = self.clone();
        this.passed = Some(jobs.to_vec());
        this
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
