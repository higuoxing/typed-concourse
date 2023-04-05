use std::error::Error;
use typed_concourse::cook;
use typed_concourse::job::Job;
use typed_concourse::pipeline::Pipeline;
use typed_concourse::resource::Resource;
use typed_concourse::task::{Command, Task};

fn main() -> Result<(), Box<dyn Error>> {
    let some_git_repo = Resource::git("https://github.com/greenplum-db/gpdb", "main");

    let pipeline = Pipeline::new()
        .with_background(
            "https://raw.githubusercontent.com/greenplum-db/gpdb/main/logo-greenplum.svg",
        )
        .append(
            Job::new("foo").then(
                Task::new()
                    .with_name("hello-world")
                    .with_input(&some_git_repo.as_task_input_resource())
                    .run(&Command::new("echo", &vec!["hello, world"]))
                    .to_step(),
            ),
        );

    match cook::cook_pipeline(&pipeline) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
