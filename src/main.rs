use std::error::Error;
use typed_concourse::cook;
use typed_concourse::job::Job;
use typed_concourse::pipeline::Pipeline;
use typed_concourse::resource::Resource;
use typed_concourse::task::{Command, Task};

fn main() -> Result<(), Box<dyn Error>> {
    let some_git_repo = Resource::git("https://github.com/higuoxing/clang-plugins", "main");

    let pipeline = Pipeline::new()
        .append(
            Job::new("foo")
                .parallel(&vec![
                    some_git_repo.as_get_resource().get_as("repo1"),
                    some_git_repo.as_get_resource().get_as("repo2"),
                ])
                .then(
                    Task::linux()
                        .with_name("hello-world")
                        .with_input(&some_git_repo.as_task_input())
                        .mutate_task_config(|task_config| {
                            task_config
                                .with_env(&vec![("ENV_VAR1", "hello")])
                                .run(&Command::new("echo", &vec!["$ENV_VAR1"]))
                        })
                        .to_step(),
                ),
        )
        .append(
            Job::new("bar")
                .then(some_git_repo.as_get_resource().get_as("foooooooo"))
                .then(Task::linux().with_name("hello").to_step()),
        );

    match cook::cook_pipeline(&pipeline) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
