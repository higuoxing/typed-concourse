# Typed Concourse [![Build and Test](https://github.com/higuoxing/typed-concourse/actions/workflows/rust.yml/badge.svg)](https://github.com/higuoxing/typed-concourse/actions/workflows/rust.yml)

> `typed-concourse` is a Rust library that allows users to write [Concourse CI](https://concourse-ci.org/) configuration in Rust.

## Features

- Type-safe configuration: `typed-concourse` provides a type-safe way to define your Concourse pipelines and tasks, ensuring that your configuration is correct at compile-time.

- Rust-based DSL: With `typed-concourse`, you can define your Concourse configuration using a Rust-based domain-specific language (DSL). This makes it easy to write and maintain complex configurations in a language you already know and love.

- Intuitive API: The API for `typed-concourse` is designed to be intuitive and easy to use. Whether you're a seasoned Rust developer or new to the language, you'll be able to use `typed-concourse` to create your Concourse pipelines and tasks quickly and easily.

- Code indention: When writing configurations in YAML/YAML generator, you still need to carefully indent your codes or the parser cannot parse it correctly. With `typed-concourse`, you can write configuration in the Rust language and your IDE/editor will take care of it!

- Jump to symbols definition: Rust IDE is very good at indexing Rust codes. With it you will gain better development experience than writing YAML codes!

## Getting started

To use `typed-concourse` in your Rust project, simply add the following to your `Cargo.toml` file:

```toml
[dependencies]
typed-concourse = { git = "https://github.com/higuoxing/typed-concourse", branch = "main" }
```

Then, in your Rust code, you can use `typed-concourse` to define your Concourse configuration. For example, here's how you could define a simple pipeline:

```rust
fn hello_world_example() {
    let pipeline = Pipeline::new().append(
        Job::new("job").with_public(true).then(
            Task::new()
                .with_name("simple-task")
                .run(&Command::new("echo", &vec!["Hello world!"]))
                .to_step(),
        ),
    );

    assert_eq!(
        cook::cook_pipeline(&pipeline).unwrap().as_str(),
        r#"jobs:
- name: job
  public: true
  plan:
  - task: simple-task
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: echo
        args:
        - Hello world!
"#
    );
}
```

This creates a pipeline with a single job that runs a simple task to echo `"Hello, world!"`.

For more examples and documentation, please refer to the [`src/example.rs`](https://github.com/higuoxing/typed-concourse/blob/main/src/examples.rs).

## Contributing

We welcome contributions to `typed-concourse`! If you'd like to contribute, please see the contributing guide for more information.

## License

`typed-concourse` is licensed under the MIT license. See [](./LICENSE) for more information.
