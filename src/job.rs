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

    pub fn with_public(&self, is_public: bool) -> Self {
        let mut this = self.clone();
        this.public = Some(is_public);
        this
    }

    pub fn with_serial(&self, is_serial: bool) -> Self {
        let mut this = self.clone();
        this.serial = Some(is_serial);
        this
    }

    pub fn then(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.plan.push(step);
        this
    }

    pub fn try_(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.plan.push(Step::Try(Try {
            try_: Box::new(step),
        }));
        this
    }

    pub fn parallel(&self, steps: &[Step]) -> Self {
        let mut this = self.clone();
        this.plan
            .push(Step::InParallel(InParallel::Steps(steps.to_vec())));
        this
    }

    pub fn on_failure(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.on_failure = Some(step);
        this
    }

    pub fn on_error(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.on_error = Some(step);
        this
    }

    pub fn on_abort(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.on_abort = Some(step);
        this
    }

    pub fn on_success(&self, step: Step) -> Self {
        let mut this = self.clone();
        this.on_success = Some(step);
        this
    }

    pub fn fallible(
        &self,
        on_failure: Option<Step>,
        on_error: Option<Step>,
        on_abort: Option<Step>,
    ) -> Self {
        if on_failure.is_none() && on_error.is_none() && on_abort.is_none() {
            panic!(
                "One of on_failure, on_error, on_abort hooks must be specified in fallible jobs."
            );
        }
        let mut this = self.clone();
        this.on_failure = on_failure;
        this.on_error = on_error;
        this.on_abort = on_abort;
        this
    }

    pub fn ensure(&self, ensure: Step) -> Self {
        let mut this = self.clone();
        this.ensure = Some(ensure);
        this
    }

    pub fn plan(&self) -> &[Step] {
        &self.plan
    }

    pub fn bind(self, var: &mut Self) -> Self {
        *var = self.clone();
        self
    }
}
