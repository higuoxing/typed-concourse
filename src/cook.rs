use crate::errors::Errors;
use crate::pipeline::Pipeline;
use crate::resource::Resource;
use crate::resource::ResourceTypes;
use crate::step::InParallel;
use crate::step::Step;
use crate::task::Input;
use crate::task::Output;
use crate::task::TaskDef;
use crate::task::TaskResource;
use serde_yaml;
use std::collections::BTreeMap;

fn collect_resource_in_step(
    step: &Step,
    curr_resources: &mut BTreeMap<String, Resource>,
    resource_collector: &mut BTreeMap<String, Resource>,
) -> Result<Vec<Step>, Errors> {
    let mut adjusted_step = step.clone();
    let mut parallel_to_get = vec![];
    match step {
        Step::Get(ref get_step) => {
            curr_resources.insert(get_step.get.clone(), get_step.resource.clone());
        }
        Step::InParallel(ref in_parallel) => match in_parallel {
            InParallel::Steps(ref steps) => {
                let mut temp_curr_resources = BTreeMap::new();
                let mut adjusted_parallel_steps = vec![];
                for parallel_step in steps.iter() {
                    adjusted_parallel_steps.append(&mut collect_resource_in_step(
                        parallel_step,
                        &mut temp_curr_resources,
                        resource_collector,
                    )?);
                }

                temp_curr_resources
                    .iter()
                    .map(|(k, v)| curr_resources.insert(k.clone(), v.clone()))
                    .count();

                adjusted_step = Step::InParallel(InParallel::Steps(adjusted_parallel_steps));
            }
            _ => {}
        },
        Step::Put(..) => {}
        Step::Task(ref task_step) => {
            let mut inputs_for_new_config = vec![];
            let mut outputs_for_new_config = vec![];
            // 1. Check if we need to get resource for inputs.
            if let Some(ref inputs) = task_step.inputs {
                for inp in inputs.iter() {
                    if let TaskResource::Resource {
                        ref resource,
                        ref get_as,
                        ..
                    } = inp
                    {
                        if !curr_resources.contains_key(resource.name().as_str()) {
                            curr_resources.insert(resource.name(), resource.clone());
                            match get_as {
                                Some(ref get_as_name) => parallel_to_get
                                    .push(resource.as_get_resource().get_as(get_as_name.as_str())),
                                None => parallel_to_get.push(resource.as_get_resource().get()),
                            }

                            inputs_for_new_config.push(Input::new(resource.name().as_str()));
                        }
                    } else if let TaskResource::Output { ref name, .. } = inp {
                        inputs_for_new_config.push(Input::new(name.as_str()));
                    }
                }
            }

            // Append outputs to task_config.
            if let Some(ref outputs) = task_step.outputs {
                for out in outputs.iter() {
                    if let TaskResource::Output { ref name, .. } = out {
                        outputs_for_new_config.push(Output::new(name.as_str()));
                    }
                }
            }

            match task_step.task_def {
                TaskDef::File { .. } => { /* Do nothing??? */ }
                TaskDef::Config { .. } => {
                    adjusted_step =
                        Step::Task(task_step.clone().mutate_task_config(|task_config| {
                            let mut new_task_config = task_config.clone();
                            if !inputs_for_new_config.is_empty() {
                                new_task_config.inputs = Some(inputs_for_new_config.clone());
                            }
                            if !outputs_for_new_config.is_empty() {
                                new_task_config.outputs = Some(outputs_for_new_config.clone());
                            }
                            new_task_config
                        }));
                }
            }

            // 2. Check if we need to get resource for task.image.
            if let Some(image) = task_step.image.as_ref() {
                if !curr_resources.contains_key(image.resource.name.as_str()) {
                    curr_resources.insert(image.resource.name(), image.resource.clone());
                    parallel_to_get.push(
                        task_step
                            .image
                            .as_ref()
                            .unwrap()
                            .resource
                            .as_get_resource()
                            .get(),
                    );
                }
            }
        }
    }

    curr_resources
        .iter()
        .map(|(k, v)| resource_collector.insert(k.clone(), v.clone()))
        .count();

    if !parallel_to_get.is_empty() {
        Ok(vec![
            Step::InParallel(InParallel::Steps(parallel_to_get)),
            adjusted_step,
        ])
    } else {
        Ok(vec![adjusted_step])
    }
}

fn collect_resource(
    pipeline: &Pipeline,
    resource_collector: &mut BTreeMap<String, Resource>,
) -> Result<Pipeline, Errors> {
    let mut adjusted_pipeline = pipeline.clone();
    // Reset the plan, since we will reconstruct it.
    adjusted_pipeline.jobs = vec![];

    for job in pipeline.jobs.iter() {
        // Current resources is used to record available resources for the current step.
        let mut curr_job = job.clone();
        // Reset the plan, since we will reconstruct it.
        curr_job.plan = vec![];

        let mut curr_resources = BTreeMap::new();
        for step in job.plan.iter() {
            let mut adjusted_steps =
                collect_resource_in_step(step, &mut curr_resources, resource_collector)?;
            curr_job.plan.append(&mut adjusted_steps);
        }

        // Append the adjusted job to the pipeline.
        adjusted_pipeline.jobs.push(curr_job);
    }

    Ok(adjusted_pipeline)
}

fn optimize_pipeline(pipeline: &Pipeline) -> Result<Pipeline, Errors> {
    let mut resource_collector = BTreeMap::new();
    let pipeline = collect_resource(pipeline, &mut resource_collector)?;
    Ok(pipeline
        .with_resources(resource_collector.iter().map(|(_, v)| v.clone()).collect())
        .with_resource_types(
            resource_collector
                .iter()
                .filter_map(|(_, v)| {
                    if let ResourceTypes::Custom { .. } = v.type_ {
                        Some((v.type_.to_string(), v.type_.clone()))
                    } else {
                        None
                    }
                })
                .collect::<BTreeMap<String, ResourceTypes>>()
                .into_iter()
                .map(|(_, v)| v.clone())
                .collect(),
        ))
}

pub fn cook_pipeline(pipeline: &Pipeline) -> Result<String, Errors> {
    let pipeline = optimize_pipeline(pipeline)?;
    match serde_yaml::to_string(&pipeline) {
        Ok(yaml) => Ok(yaml),
        Err(e) => Err(Errors::SerdeError(e)),
    }
}
