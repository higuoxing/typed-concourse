use crate::errors::Errors;
use crate::pipeline::Pipeline;
use crate::resource::Resource;
use crate::step::InParallel;
use crate::step::Step;
use crate::task::Input;
use crate::task::Output;
use crate::task::TaskDef;
use crate::task::TaskResource;
use serde_yaml;
use std::collections::HashMap;

fn collect_resource_in_step(
    step: &Step,
    curr_resources: &mut HashMap<String, Resource>,
    resource_collector: &mut HashMap<String, Resource>,
) -> Result<Vec<Step>, Errors> {
    let mut adjusted_step = step.clone();
    let mut parallel_to_get = vec![];
    match step {
        Step::Get(ref get_step) => {
            curr_resources.insert(get_step.get.clone(), get_step.resource.clone());
        }
        Step::InParallel(ref in_parallel) => match in_parallel {
            InParallel::Steps(ref steps) => {
                let mut temp_curr_resources = HashMap::new();
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
            match task_step.task_def {
                TaskDef::File { .. } => { /* Do nothing. */ }
                TaskDef::Config {
                    ref config,
                    ref inputs,
                    ref outputs,
                } => {
                    let mut new_config = config.clone();
                    // 1. Check if we need to get resource for inputs.
                    if let Some(ref inputs) = inputs {
                        for inp in inputs.iter() {
                            if let TaskResource::Resource(ref res) = inp {
                                if !curr_resources.contains_key(res.name().as_str()) {
                                    curr_resources.insert(res.name(), res.clone());
                                    parallel_to_get.push(res.as_get_resource().get());
                                    if config.inputs.is_none() {
                                        new_config.inputs =
                                            Some(vec![Input::new(res.name().as_str())]);
                                    } else {
                                        let mut new_inputs = config.inputs.clone().unwrap();
                                        new_inputs.push(Input::new(res.name().as_str()));
                                        new_config.inputs = Some(new_inputs);
                                    }
                                }
                            } else if let TaskResource::Output(ref output) = inp {
                                if config.inputs.is_none() {
                                    new_config.inputs = Some(vec![Input::new(output.as_str())]);
                                } else {
                                    let mut new_inputs = config.inputs.clone().unwrap();
                                    new_inputs.push(Input::new(output.as_str()));
                                    new_config.inputs = Some(new_inputs);
                                }
                            }
                        }
                    }

                    // Append outputs to task_config.
                    if let Some(ref outputs) = outputs {
                        for out in outputs.iter() {
                            if let TaskResource::Output(ref out) = out {
                                if config.outputs.is_none() {
                                    new_config.outputs = Some(vec![Output::new(out.as_str())])
                                } else {
                                    let mut new_outputs = config.outputs.clone().unwrap();
                                    new_outputs.push(Output::new(out.as_str()));
                                    new_config.outputs = Some(new_outputs);
                                }
                            }
                        }
                    }

                    adjusted_step =
                        Step::Task(task_step.clone().mutate_task_config(|_| new_config.clone()));

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
    resource_collector: &mut HashMap<String, Resource>,
) -> Result<Pipeline, Errors> {
    let mut adjusted_pipeline = pipeline.clone();
    // Reset the plan, since we will reconstruct it.
    adjusted_pipeline.jobs = vec![];

    for job in pipeline.jobs.iter() {
        // Current resources is used to record available resources for the current step.
        let mut curr_job = job.clone();
        // Reset the plan, since we will reconstruct it.
        curr_job.plan = vec![];

        let mut curr_resources = HashMap::new();
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
    let mut resource_collector = HashMap::new();
    let pipeline = collect_resource(pipeline, &mut resource_collector)?;
    Ok(pipeline.with_resources(resource_collector.iter().map(|(_, v)| v.clone()).collect()))
}

pub fn cook_pipeline(pipeline: &Pipeline) -> Result<String, Errors> {
    let pipeline = optimize_pipeline(pipeline)?;
    match serde_yaml::to_string(&pipeline) {
        Ok(yaml) => Ok(yaml),
        Err(e) => Err(Errors::SerdeError(e)),
    }
}
