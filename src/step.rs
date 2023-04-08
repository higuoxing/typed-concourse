use crate::get::Get;
use crate::put::Put;
use crate::schema::Number;
use crate::task::Task;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::Serializer;

#[derive(Debug, Clone)]
pub enum InParallel {
    Steps(Vec<Step>),
    InParallelConfig {
        steps: Vec<Step>,
        limit: Option<Number>,
        fail_fast: bool,
    },
}

impl Serialize for InParallel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            InParallel::Steps(ref steps) => {
                let mut state = serializer.serialize_struct("InParallel", 1)?;
                state.serialize_field("in_parallel", steps)?;
                state.end()
            }
            InParallel::InParallelConfig { .. } => todo!(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Step {
    Get(Get),
    Put(Put),
    Task(Task),
    InParallel(InParallel),
}
