use crate::schema::Config;
use crate::schema::Identifier;
use serde::Serialize;

#[derive(Serialize)]
pub struct ResourceType {
    name: Identifier,
    #[serde(rename(serialize = "type"))]
    type_: Identifier,
    source: Config,
}

#[derive(Serialize)]
pub struct Resource {
    name: Identifier,
    #[serde(rename(serialize = "type"))]
    type_: Identifier,
    source: Config,
}

#[derive(Serialize)]
pub struct AnonymousResource {
    #[serde(rename(serialize = "type"))]
    type_: Identifier,
    source: Config,
}
