use crate::schema::Identifier;
use crate::step::InParallel;
use crate::step::Step;
use crate::step::Try;
use serde::Serialize;

#[derive(Debug, Clone)]
pub(crate) enum JobKind {
    Unbound,
    Initialized,
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    #[serde(skip_serializing)]
    pub(crate) kind: JobKind,
    pub(crate) name: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) serial: Option<bool>,
    pub(crate) plan: Vec<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) on_failure: Option<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) on_error: Option<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) on_abort: Option<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) on_success: Option<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) ensure: Option<Step>,
}

impl Job {
    pub fn unbound() -> Self {
        let mut this = Self::new("");
        this.kind = JobKind::Unbound;
        this
    }

    pub fn new(name: &str) -> Self {
        Self {
            kind: JobKind::Initialized,
            name: name.to_string(),
            public: None,
            serial: None,
            plan: vec![],
            on_failure: None,
            on_error: None,
            on_abort: None,
            on_success: None,
            ensure: None,
        }
    }

    pub fn new_fallible(
        name: &str,
        on_failure: Option<Step>,
        on_error: Option<Step>,
        on_abort: Option<Step>,
    ) -> Self {
        Self::new(name).fallible(on_failure, on_error, on_abort)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn with_public(mut self, is_public: bool) -> Self {
        self.public = Some(is_public);
        self
    }

    pub fn with_serial(mut self, is_serial: bool) -> Self {
        self.serial = Some(is_serial);
        self
    }

    pub fn then(mut self, step: Step) -> Self {
        self.plan.push(step);
        self
    }

    pub fn try_(mut self, step: Step) -> Self {
        self.plan.push(Step::Try(Try {
            try_: Box::new(step),
        }));
        self
    }

    pub fn parallel(mut self, steps: &[Step]) -> Self {
        self.plan
            .push(Step::InParallel(InParallel::Steps(steps.to_vec())));
        self
    }

    pub fn on_failure(mut self, step: Step) -> Self {
        self.on_failure = Some(step);
        self
    }

    pub fn on_error(mut self, step: Step) -> Self {
        self.on_error = Some(step);
        self
    }

    pub fn on_abort(mut self, step: Step) -> Self {
        self.on_abort = Some(step);
        self
    }

    pub fn on_success(mut self, step: Step) -> Self {
        self.on_success = Some(step);
        self
    }

    pub fn fallible(
        mut self,
        on_failure: Option<Step>,
        on_error: Option<Step>,
        on_abort: Option<Step>,
    ) -> Self {
        if on_failure.is_none() && on_error.is_none() && on_abort.is_none() {
            panic!(
                "One of on_failure, on_error, on_abort hooks must be specified in fallible jobs."
            );
        }
        self.on_failure = on_failure;
        self.on_error = on_error;
        self.on_abort = on_abort;
        self
    }

    pub fn ensure(mut self, ensure: Step) -> Self {
        self.ensure = Some(ensure);
        self
    }

    pub fn plan(&self) -> &[Step] {
        &self.plan
    }

    pub fn bind(self, var: &mut Self) -> Self {
        *var = self.clone();
        self
    }
}
