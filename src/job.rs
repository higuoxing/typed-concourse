use crate::errors::Errors;
use crate::schema::Identifier;
use crate::step::Step;
use serde::Serialize;

#[derive(Serialize)]
pub struct Job {
    name: Identifier,
    plan: Vec<Step>,
}

impl Job {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            plan: vec![],
        }
    }

    pub fn append(mut self, step: Step) -> Result<Self, Errors> {
        self.plan.push(step);
        Ok(self)
    }
}
