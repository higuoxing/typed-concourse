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
    pub(crate) fn from(identifier: &str, resource: &Resource, version: Option<Version>) -> Self {
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

    pub fn with_trigger(mut self, trigger: bool) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn with_passed(mut self, jobs: &[Job]) -> Self {
        self.passed = Some(jobs.to_vec());
        self
    }

    pub fn get(self) -> Step {
        Step::Get(self)
    }

    pub fn get_as(mut self, alias: &str) -> Step {
        self.get = alias.to_string();
        Step::Get(self)
    }
}
