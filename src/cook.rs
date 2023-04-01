use crate::errors::Errors;
use crate::pipeline::Pipeline;
use serde_yaml;

fn check_pipeline(pipeline: &Pipeline) -> Result<(), Errors> {
    if pipeline.jobs().is_empty() {
        return Err(Errors::CookError(String::from(
            "Pipeline must contain at least one job",
        )));
    }

    Ok(())
}

pub fn cook_pipeline(pipeline: &Pipeline) -> Result<String, Errors> {
    // Verify the pipeline before generating the YAML configuration file.
    check_pipeline(pipeline)?;

    match serde_yaml::to_string(pipeline) {
        Ok(yaml) => Ok(yaml),
        Err(e) => Err(Errors::SerdeError(e)),
    }
}
