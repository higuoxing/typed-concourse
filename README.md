# Typed Concourse

> `typed-concourse` is a Rust library that allows users to write Concourse CI configuration in Rust.

## Features

- Type-safe configuration: `typed-concourse` provides a type-safe way to define your Concourse pipelines and tasks, ensuring that your configuration is correct at compile-time.

- Rust-based DSL: With `typed-concourse`, you can define your Concourse configuration using a Rust-based domain-specific language (DSL). This makes it easy to write and maintain complex configurations in a language you already know and love.

- Intuitive API: The API for `typed-concourse` is designed to be intuitive and easy to use. Whether you're a seasoned Rust developer or new to the language, you'll be able to use `typed-concourse` to create your Concourse pipelines and tasks quickly and easily.

## Getting started

To use `typed-concourse` in your Rust project, simply add the following to your `Cargo.toml` file:

```toml
[dependencies]
typed-concourse = "0.1.0"
```

Then, in your Rust code, you can use `typed-concourse` to define your Concourse configuration. For example, here's how you could define a simple pipeline:

```rust
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
```

This creates a pipeline with a single job that runs a simple task to echo `"Hello, world!"`.

For more examples and documentation, please see the API documentation.

## Contributing

We welcome contributions to `typed-concourse`! If you'd like to contribute, please see the contributing guide for more information.

## License

`typed-concourse` is licensed under the MIT license. See [](./LICENSE) for more information.
