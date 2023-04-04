use crate::errors::Errors;
use crate::pipeline::Pipeline;
use crate::resource::Resource;
use crate::resource::ResourceTypes;
use crate::step::InParallel;
use crate::step::Step;
use crate::task::Input;
use crate::task::TaskResource;
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
    resource_types: &mut HashMap<String, ResourceTypes>,
) -> Result<(), Errors> {
    match step {
        Step::Get(ref get_step) => {
            if let ResourceTypes::Custom(_) = get_step.resource().resource_type().clone() {
                resource_types.insert(
                    get_step.resource().resource_type().to_string(),
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
        Step::Task(ref task) => match task.inputs() {
            Some(ref inputs) => {
                for inp in inputs {
                    if let TaskResource::Resource(ref resource) = inp {
                        resources.insert(resource.name(), resource.clone());
                    }
                }
            }
            None => {}
        },
        _ => todo!(),
    }
    Ok(())
}

fn collect_resource_used_in_task(step: &Step) -> Result<(Option<Vec<Resource>>, Step), Errors> {
    match step {
        Step::Task(ref task) => match task.inputs() {
            Some(ref inputs) => {
                let mut resources = vec![];
                for inp in inputs {
                    if let TaskResource::Resource(ref resource) = inp {
                        resources.push(resource.clone());
                    }
                }

                let resources = if resources.is_empty() {
                    None
                } else {
                    Some(resources)
                };

                match &resources {
                    Some(res) => {
                        // Mutate the task to add 'inputs' tags into its config.
                        let mut task = task.clone();
                        match task.task_config_mut().inputs {
                            Some(ref mut inputs) => {
                                inputs.append(
                                    &mut res
                                        .iter()
                                        .map(|r| Input::new(r.name().as_str()))
                                        .collect(),
                                );
                            }
                            None => {
                                task.task_config_mut().inputs = Some(
                                    res.iter().map(|r| Input::new(r.name().as_str())).collect(),
                                );
                            }
                        }
                        return Ok((resources, Step::Task(task)));
                    }
                    None => { /* Do nothing. */ }
                }

                Ok((resources, step.clone()))
            }
            None => Ok((None, step.clone())),
        },
        _ => Ok((None, step.clone())),
    }
}

fn optimize_pipeline(pipeline: &Pipeline) -> Result<Pipeline, Errors> {
    let mut new_pipeline = pipeline.clone();
    new_pipeline.jobs = vec![];

    for job in pipeline.jobs() {
        let mut new_job = job.clone();
        new_job.reset_plan();

        for step in job.plan() {
            // 1. Identify resources that are directly referenced in tasks.
            // 2. Insert get steps before the task and add 'input' tags to that task.
            let (ref_resources, step) = collect_resource_used_in_task(step)?;
            match ref_resources {
                Some(resources) => {
                    new_job = new_job.parallel(
                        &resources
                            .iter()
                            .map(|res| res.as_get_resource().get())
                            .collect::<Vec<Step>>(),
                    );
                }
                None => {}
            }
            new_job = new_job.then(step);
        }

        new_pipeline = new_pipeline.append(new_job);
    }

    Ok(new_pipeline)
}

fn collect_resources(pipeline: &Pipeline) -> Result<Pipeline, Errors> {
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
        .map(|(_, v)| v.clone())
        .collect::<Vec<ResourceTypes>>();

    Ok(pipeline
        .with_resources(resources)
        .with_resource_types(resource_types))
}

pub fn cook_pipeline(pipeline: &Pipeline) -> Result<String, Errors> {
    let pipeline = optimize_pipeline(pipeline)?;
    // Collect resources.
    let pipeline = collect_resources(&pipeline)?;

    // Verify the pipeline before generating the YAML configuration file.
    check_pipeline(&pipeline)?;

    match serde_yaml::to_string(&pipeline) {
        Ok(yaml) => Ok(yaml),
        Err(e) => Err(Errors::SerdeError(e)),
    }
}
