//! The context / state for the server

use std::sync::RwLock;
use crate::config::Config;
use tera::Tera;

pub struct ServerContext {
    pub config: Config,
    pub tera: RwLock<Tera>,
}

