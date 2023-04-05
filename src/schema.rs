use std::collections::HashMap;

use serde::Serialize;

// https://concourse-ci.org/config-basics.html#schema.identifier
pub type Identifier = String;
// https://concourse-ci.org/config-basics.html#schema.file-path
pub type FilePath = String;
// https://concourse-ci.org/config-basics.html#schema.dir-path
pub type DirPath = String;
// https://concourse-ci.org/config-basics.html#schema.config
pub type Config = HashMap<String, String>;
// https://concourse-ci.org/config-basics.html#schema.vars
pub type Vars = HashMap<String, String>;
// https://concourse-ci.org/config-basics.html#schema.env-vars
pub type EnvVars = HashMap<String, String>;
// https://concourse-ci.org/config-basics.html#schema.version
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Version {
    Latest,
    Every,
    Custom(HashMap<String, String>),
}
// https://concourse-ci.org/config-basics.html#schema.number
pub type Number = i64;
