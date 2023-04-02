use crate::errors::Errors;
use crate::pipeline::Pipeline;
use crate::resource::Resource;
use crate::resource::ResourceType;
use crate::step::InParallel;
use crate::step::Step;
use serde_yaml;
use std::collections::HashMap;

fn check_pipeline(pipeline: &Pipeline) -> Result<(), Errors> {
    if pipeline.jobs().is_empty() {
        return Err(Errors::CookError(String::from(
            "Pipeline must contain at least one job",
        )));
    }

    Ok(())
}

fn collect_resource(
    step: &Step,
    resources: &mut HashMap<String, Resource>,
    resource_types: &mut HashMap<String, ResourceType>,
) -> Result<(), Errors> {
    match step {
        Step::Get(ref get_step) => {
            if get_step.resource().resource_type().type_() != String::from("core") {
                resource_types.insert(
                    get_step.resource().resource_type().name(),
                    get_step.resource().resource_type().clone(),
                );
            }
            resources.insert(get_step.resource().name(), get_step.resource().clone());
        }
        Step::InParallel(ref in_parallel) => match in_parallel {
            InParallel::Steps(ref steps) => {
                for s in steps {
                    collect_resource(s, resources, resource_types)?;
                }
            }
            _ => todo!(),
        },
        Step::Task(_) => return Ok(()),
        _ => todo!(),
    }
    Ok(())
}

fn initialize_resources(pipeline: &Pipeline) -> Result<Pipeline, Errors> {
    let mut resource_types = HashMap::new();
    let mut resources = HashMap::new();

    for job in pipeline.jobs() {
        for step in job.plan() {
            collect_resource(step, &mut resources, &mut resource_types)?;
        }
    }

    let resources = resources.iter().map(|(_, res)| res.clone()).collect();
    let resource_types = resource_types
        .iter()
        .map(|(_, res_type)| res_type.clone())
        .collect();

    let this = pipeline.clone();
    this.with_resources(resources)?
        .with_resource_types(resource_types)
}

pub fn cook_pipeline(pipeline: &Pipeline) -> Result<String, Errors> {
    // Collect resources.
    let new_pipeline = initialize_resources(pipeline)?;

    // Verify the pipeline before generating the YAML configuration file.
    check_pipeline(&new_pipeline)?;

    match serde_yaml::to_string(&new_pipeline) {
        Ok(yaml) => Ok(yaml),
        Err(e) => Err(Errors::SerdeError(e)),
    }
}
