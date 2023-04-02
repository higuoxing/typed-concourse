use crate::core::Identifier;
use crate::errors::Errors;
use crate::step::InParallelStep;
use crate::step::Step;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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

    pub fn then(mut self, step: Step) -> Result<Self, Errors> {
        self.plan.push(step);
        Ok(self)
    }

    pub fn parallel(mut self, steps: &Vec<Step>) -> Result<Self, Errors> {
        self.plan
            .push(Step::InParallel(InParallelStep::Steps(steps.clone())));
        Ok(self)
    }

    pub fn plan(&self) -> &Vec<Step> {
        &self.plan
    }
}
