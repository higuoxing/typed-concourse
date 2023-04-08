use crate::resource::Resource;
use crate::schema::Identifier;
use crate::step::Step;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Put {
    pub(crate) put: Identifier,
    pub(crate) resource: Resource,
    pub(crate) params: BTreeMap<String, String>,
}

impl Serialize for Put {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("PutStep", 5)?;
        if !self.put.is_empty() {
            state.serialize_field("put", &self.put)?;
            state.serialize_field("resource", &self.resource.name())?;
        } else {
            state.serialize_field("put", &self.resource.name())?;
        }

        if !self.params.is_empty() {
            state.serialize_field("params", &self.params)?;
        }

        state.end()
    }
}

impl Put {
    pub(crate) fn from(identifier: &str, resource: &Resource) -> Self {
        Self {
            put: identifier.to_string(),
            resource: resource.clone(),
            params: BTreeMap::new(),
        }
    }

    pub fn with_params(&self, params: &[(&str, &str)]) -> Self {
        let mut this = self.clone();
        this.params = params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        this
    }

    pub fn put(self) -> Step {
        Step::Put(self)
    }

    pub fn put_as(self, name: &str) -> Step {
        let mut this = self;
        this.put = name.to_string();
        Step::Put(this)
    }
}
