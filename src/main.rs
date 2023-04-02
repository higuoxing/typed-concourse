use std::error::Error;
use typed_concourse::cook;
use typed_concourse::core::Version;
use typed_concourse::job::Job;
use typed_concourse::pipeline::Pipeline;
use typed_concourse::resource::git_resource;

fn main() -> Result<(), Box<dyn Error>> {
    let some_git_repo = git_resource(
        "gppkg",
        "https://github.com/higuoxing/clang-plugins",
        "main",
    );

    let some_git_repo2 = git_resource(
        "timestamp9",
        "https://github.com/higuoxing/timestamp9.git",
        "master",
    );

    let pipeline = Pipeline::new()
        .append(Job::new("foo").parallel(&vec![
            some_git_repo.get().trigger_new_build()?,
            some_git_repo2.get().trigger_new_build()?.with_version(Version::Latest)?,
            some_git_repo2.get_as("ts9").trigger_new_build()?.with_version(Version::Every)?,
        ])?)?
        .append(Job::new("bar"))?;

    match cook::cook_pipeline(&pipeline) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
