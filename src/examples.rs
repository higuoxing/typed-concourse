#[cfg(test)]
mod examples {
    use crate::{
        cook::{self, cook_pipeline},
        job::Job,
        pipeline::Pipeline,
        resource::{Resource, ResourceTypes},
        task::{Command, Task, TaskResource},
    };

    // https://concourse-ci.org/hello-world-example.html
    #[test]
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

    // https://concourse-ci.org/serial-job-example.html
    #[test]
    fn serial_job_example() {
        let pipeline = Pipeline::new().append(
            Job::new("job").with_public(true).with_serial(true).then(
                Task::new()
                    .with_name("simple-task")
                    .mutate_task_config(|task_config| {
                        task_config.run(&Command::new("echo", &vec!["Hello world!"]))
                    })
                    .to_step(),
            ),
        );

        assert_eq!(
            cook::cook_pipeline(&pipeline).unwrap().as_str(),
            r#"jobs:
- name: job
  public: true
  serial: true
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

    // https://concourse-ci.org/pipeline-vars-example.html
    // Variables should be load by the fly command.
    // We don't process it in our library.
    #[test]
    fn pipeline_vars_example() {
        let pipeline = Pipeline::new()
            .append(
                Job::new("((first))-job").with_public(true).then(
                    Task::new()
                        .with_name("simple-task")
                        .run(&Command::new("echo", &vec!["Hello, ((hello))!"]))
                        .to_step(),
                ),
            )
            .append(
                Job::new("level-((number))-job").with_public(true).then(
                    Task::new()
                        .with_name("simple-task")
                        .run(&Command::new("echo", &vec!["Hello, ((hello))!"]))
                        .to_step(),
                ),
            );

        assert_eq!(
            cook::cook_pipeline(&pipeline).unwrap().as_str(),
            r#"jobs:
- name: ((first))-job
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
        - Hello, ((hello))!
- name: level-((number))-job
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
        - Hello, ((hello))!
"#
        );
    }

    // https://concourse-ci.org/time-trigger-example.html
    #[test]
    fn time_trigger_example() {
        let every_30s = Resource::time("30s");
        let pipeline = Pipeline::new().append(
            Job::new("job")
                .with_public(true)
                .then(every_30s.as_get_resource().with_trigger(true).get())
                .then(
                    Task::new()
                        .with_name("simple-task")
                        .run(&Command::new("echo", &vec!["Hello, world!"]))
                        .to_step(),
                ),
        );

        assert_eq!(
            cook::cook_pipeline(&pipeline).unwrap().as_str(),
            r#"jobs:
- name: job
  public: true
  plan:
  - get: every-30s
    trigger: true
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
        - Hello, world!
resources:
- name: every-30s
  type: time
  icon: clock-outline
  source:
    interval: 30s
"#
        );
    }

    // https://concourse-ci.org/git-trigger-example.html
    #[test]
    fn git_trigger_example() {
        let concourse_docs_git = Resource::git("https://github.com/concourse/docs", "")
            .with_name("concourse-docs-git")
            .with_trigger(true);
        let pipeline = Pipeline::new().append(
            Job::new("job").then(
                Task::new()
                    .with_name("list-files")
                    .with_inputs(&vec![&concourse_docs_git.as_task_input_resource()])
                    .run(&Command::new("ls", &vec!["./concourse-docs-git"]))
                    .to_step(),
            ),
        );

        assert_eq!(
            cook::cook_pipeline(&pipeline).unwrap(),
            r#"jobs:
- name: job
  plan:
  - in_parallel:
    - get: concourse-docs-git
      trigger: true
  - task: list-files
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: ls
        args:
        - ./concourse-docs-git
      inputs:
      - name: concourse-docs-git
resources:
- name: concourse-docs-git
  type: git
  icon: github
  source:
    uri: https://github.com/concourse/docs
"#
        );
    }

    // https://concourse-ci.org/hooks-example.html
    #[test]
    fn hooks_example() {
        let echo = |what: &str, state: &str| {
            Command::new(
                "echo",
                &vec![format!("This {} was {}!", what, state).as_str()],
            )
        };

        let pipeline = Pipeline::new().append(
            Job::new("job")
                .with_public(true)
                .on_success(
                    Task::new()
                        .with_name("job-success")
                        .run(&echo("job", "succeeded"))
                        .to_step(),
                )
                .on_error(
                    Task::new()
                        .with_name("job-failure")
                        .run(&echo("job", "failed"))
                        .to_step(),
                )
                .on_abort(
                    Task::new()
                        .with_name("job-aborted")
                        .run(&echo("job", "aborted"))
                        .to_step(),
                )
                .then(
                    Task::new()
                        .with_name("successful-task")
                        .run(&Command::new("sh", &vec!["-lc", "exit 0"]))
                        .on_success(
                            Task::new()
                                .with_name("task-success")
                                .run(&echo("task", "succeeded"))
                                .to_step(),
                        )
                        .on_abort(
                            Task::new()
                                .with_name("task-aborted")
                                .run(&echo("task", "aborted"))
                                .to_step(),
                        )
                        .to_step(),
                )
                .then(
                    Task::new()
                        .with_name("failing-task")
                        .run(&Command::new("sh", &vec!["-lc", "exit 1"]))
                        .on_failure(
                            Task::new()
                                .with_name("task-failure")
                                .run(&echo("task", "failed"))
                                .to_step(),
                        )
                        .to_step(),
                ),
        );

        assert_eq!(
            cook_pipeline(&pipeline).unwrap(),
            r#"jobs:
- name: job
  public: true
  plan:
  - task: successful-task
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: sh
        args:
        - -lc
        - exit 0
    on_abort:
      task: task-aborted
      config:
        platform: linux
        image_resource:
          type: registry-image
          source:
            repository: busybox
        run:
          path: echo
          args:
          - This task was aborted!
    on_success:
      task: task-success
      config:
        platform: linux
        image_resource:
          type: registry-image
          source:
            repository: busybox
        run:
          path: echo
          args:
          - This task was succeeded!
  - task: failing-task
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: sh
        args:
        - -lc
        - exit 1
    on_failure:
      task: task-failure
      config:
        platform: linux
        image_resource:
          type: registry-image
          source:
            repository: busybox
        run:
          path: echo
          args:
          - This task was failed!
  on_error:
    task: job-failure
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: echo
        args:
        - This job was failed!
  on_abort:
    task: job-aborted
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: echo
        args:
        - This job was aborted!
  on_success:
    task: job-success
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: echo
        args:
        - This job was succeeded!
"#
        );
    }

    // https://concourse-ci.org/task-step.html
    #[test]
    fn fetching_and_using_an_image() {
        let golang_image = Resource::registry_image("golang").with_source(&[("tag", "1.17")]);
        let pipeline = Pipeline::new().append(
            Job::new("fetch-and-run-image").then(
                Task::new()
                    .with_name("use-fetched-image-in-task")
                    .with_image(golang_image.as_task_image_resource())
                    .run(&Command::new("go", &["version"]))
                    .to_step(),
            ),
        );

        assert_eq!(
            cook_pipeline(&pipeline).unwrap(),
            r#"jobs:
- name: fetch-and-run-image
  plan:
  - in_parallel:
    - get: golang
  - task: use-fetched-image-in-task
    config:
      platform: linux
      run:
        path: go
        args:
        - version
    image: golang
resources:
- name: golang
  type: registry-image
  icon: docker
  source:
    repository: golang
    tag: '1.17'
"#
        );
    }

    // https://concourse-ci.org/task-inputs-outputs-example.html
    #[test]
    fn task_inputs_outputs_example() {
        let mut files = TaskResource::unbound();
        let pipeline = Pipeline::new().append(
            Job::new("create-and-consume")
                .then(
                    Task::new()
                        .with_name("make-a-file")
                        .run(&Command::new(
                            "sh",
                            &[
                                "-exc",
                                "ls -la; echo \"Create a file on $(date)\" > ./files/created_file",
                            ],
                        ))
                        .bind_outputs(&mut [("files", &mut files)])
                        .to_step(),
                )
                .then(
                    Task::new()
                        .with_name("consume-the-file")
                        .run(&Command::new("cat", &["./files/created_file"]))
                        .with_inputs(&[&files])
                        .to_step(),
                ),
        );

        assert_eq!(
            cook_pipeline(&pipeline).unwrap(),
            r#"jobs:
- name: create-and-consume
  plan:
  - task: make-a-file
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: sh
        args:
        - -exc
        - ls -la; echo "Create a file on $(date)" > ./files/created_file
      outputs:
      - name: files
  - task: consume-the-file
    config:
      platform: linux
      image_resource:
        type: registry-image
        source:
          repository: busybox
      run:
        path: cat
        args:
        - ./files/created_file
      inputs:
      - name: files
"#
        );
    }

    // https://concourse-ci.org/task-step.html
    #[test]
    fn generic_task_input_names() {
        let repo = Resource::new("repo", &ResourceTypes::mock());
        let repo_dev = Resource::new("repo-dev", &ResourceTypes::mock());
        let ci = Resource::git("https://github.com/concourse/examples.git", "").with_name("ci");

        let pipeline = Pipeline::new().append(
            Job::new("task-input-mapping").then(
                Task::from_file("ci/tasks/generic-inputs.yml")
                    .with_name("list-inputs")
                    .with_inputs(&[
                        &ci.as_task_input_resource(),
                        &repo.as_task_input_resource().map_to("main"),
                        &repo_dev.as_task_input_resource().map_to("dev"),
                    ])
                    .to_step(),
            ),
        );

        assert_eq!(
            cook_pipeline(&pipeline).unwrap(),
            r#"jobs:
- name: task-input-mapping
  plan:
  - in_parallel:
    - get: ci
    - get: repo
    - get: repo-dev
  - task: list-inputs
    file: ci/tasks/generic-inputs.yml
    input_mapping:
      dev: repo-dev
      main: repo
resources:
- name: ci
  type: git
  icon: github
  source:
    uri: https://github.com/concourse/examples.git
- name: repo-dev
  type: mock
- name: repo
  type: mock
resource_types:
- name: mock
  type: registry-image
"#
        );
    }
}
