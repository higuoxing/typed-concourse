use std::error::Error;
use typed_concourse::cook;
use typed_concourse::job::Job;
use typed_concourse::pipeline::Pipeline;
use typed_concourse::resource::Resource;
use typed_concourse::schema::Version;
use typed_concourse::task::Task;

fn main() -> Result<(), Box<dyn Error>> {
    let some_git_repo = Resource::git("https://github.com/higuoxing/clang-plugins", "main");
    let some_git_repo2 = Resource::git("https://github.com/higuoxing/timestamp9.git", "master");

    let pipeline = Pipeline::new()
        .append(
            Job::new("foo")
                .parallel(&vec![
                    some_git_repo.as_get_resource().trigger_new_build().get(),
                    some_git_repo2
                        .as_get_resource()
                        .trigger_new_build()
                        .with_version(Version::Latest)
                        .get(),
                    some_git_repo2
                        .as_get_resource()
                        .trigger_new_build()
                        .with_version(Version::Every)
                        .get_as("ts2"),
                ])
                .then(some_git_repo.as_get_resource().get_as("gppkg2"))
                .then(Task::linux().with_name("hello-world").to_step()),
        )
        .append(Job::new("bar").then(some_git_repo.as_get_resource().get()));

    match cook::cook_pipeline(&pipeline) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
