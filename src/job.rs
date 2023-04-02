use crate::core::Identifier;
use crate::step::InParallel;
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

    pub fn then(mut self, step: Step) -> Self {
        self.plan.push(step);
        self
    }

    pub fn parallel(mut self, steps: &Vec<Step>) -> Self {
        self.plan
            .push(Step::InParallel(InParallel::Steps(steps.clone())));
        self
    }

    pub fn plan(&self) -> &Vec<Step> {
        &self.plan
    }
}
