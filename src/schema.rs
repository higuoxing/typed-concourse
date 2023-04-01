use std::collections::HashMap;

// https://concourse-ci.org/config-basics.html#schema.identifier
pub type Identifier = String;
// https://concourse-ci.org/config-basics.html#schema.file-path
pub type FilePath = String;
// https://concourse-ci.org/config-basics.html#schema.config
pub type Config = HashMap<String, String>;
// https://concourse-ci.org/config-basics.html#schema.version
pub type Version = HashMap<String, String>;
