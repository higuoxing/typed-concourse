use std::error::Error;
use typed_concourse::cook;
use typed_concourse::job::Job;
use typed_concourse::pipeline::Pipeline;
use typed_concourse::step::Step;

fn main() -> Result<(), Box<dyn Error>> {
    let pipeline = Pipeline::new()
        .append(
            Job::new("foo")
                .append(Step::get("res1").relabel("res11")?)?
                .append(Step::put("res2"))?,
        )?
        .append(Job::new("bar"))?;

    match cook::cook_pipeline(&pipeline) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
