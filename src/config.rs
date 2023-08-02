//! Configuration Module
//!

use std::{
    env,
    path::{Path, PathBuf}, 
    net::{SocketAddr, IpAddr},
};

const ROOTDIR_KEY: &str = "WEB_ROOT";
const STATICDIR_KEY: &str = "STATIC_DIR";
const TEMPLATEDIR_KEY: &str = "TEMPLATE_DIR";

const DEFAULT_ADDR: ([u8; 4], u16)  = ([0,0,0,0], 7878);

/// The config object to handle how pages are served
///
/// # Properties
/// - `rootdir` the root location of all the files to be served
/// - `staticdir` the directory that holds all "static" files
/// - `header` the file name (relative to `staticdir`) of the header to prepend to all md files
/// - `footer` the file name (relative to `staticdir`) of the footer to append to all md files
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub rootdir: PathBuf,
    pub staticdir: PathBuf,
    pub template_dir: PathBuf,
    pub addr: SocketAddr,
}

impl Config {
    pub fn build() -> ConfigBuilder {
        ConfigBuilder::new()
    }
    
}

impl Default for Config {
    fn default() -> Config {
        Config {
            addr: SocketAddr::from(DEFAULT_ADDR),
            rootdir: PathBuf::from("./"),
            staticdir: PathBuf::from("./sample/static"),
            template_dir: PathBuf::from("./sample/templates"),
        }
    }
}


/// Builder for the configuration object
///
/// Only handles setting config values from variables.
/// Reading from commandline arguments or environment must be done separately.
pub struct ConfigBuilder {
    rootdir: PathBuf,
    staticdir: PathBuf,
    template_dir: PathBuf,
    addr: SocketAddr,
}

impl ConfigBuilder {
    /// Start building a config from the defaults
    pub fn new() -> ConfigBuilder {
        let config = Config::default();
        ConfigBuilder { 
            rootdir: config.rootdir,
            staticdir: config.staticdir,
            template_dir: config.template_dir,
            addr: config.addr,
        }
    }
    
    /// Returns the finished Config
    pub fn build(self) -> Config {
        Config {
            rootdir: self.rootdir,
            staticdir: self.staticdir,
            template_dir: self.template_dir,
            addr: self.addr,
        }
    }

    /// Sources Environment variables for the config
    ///
    /// rootdir sourced rom "WEB_ROOT"
    /// staticdir sourced from "STATIC_DIR"
    pub fn source_env(mut self) -> Self {
        if let Some(rootdir) = env::var_os(ROOTDIR_KEY) {
            eprintln!("rootdir found as {:?}", rootdir);
            self.rootdir = PathBuf::from(rootdir);
        }
        if let Some(static_dir) = env::var_os(STATICDIR_KEY) {
            eprintln!("static dir found as {:?}", static_dir);
            self.staticdir = PathBuf::from(static_dir);
        }
        if let Some(template_dir) = env::var_os(TEMPLATEDIR_KEY) {
            eprintln!("template dir found as {:?}", template_dir);
            self.template_dir = PathBuf::from(template_dir);
        }
        self
    }

    /// Set the root directory for the fileserver
    pub fn set_root(&mut self, path: &Path) -> &ConfigBuilder {
        self.rootdir = PathBuf::from(path);
        self
    }

    /// Set the static directory for the fileserver
    pub fn set_static(&mut self, path: &Path) -> &ConfigBuilder{
        self.staticdir = PathBuf::from(path);
        self
    }

    /// Set the template directory for the fileserver
    pub fn set_template(&mut self, path: &Path) -> &ConfigBuilder{
        self.template_dir = PathBuf::from(path);
        self
    }

    pub fn set_address(&mut self, addr: &SocketAddr) -> &ConfigBuilder {
            self.addr = addr.clone();
            self
        }

    pub fn set_ip<T>(&mut self, new_ip: T) -> &ConfigBuilder
        where IpAddr: From<T> {
            self.addr.set_ip(IpAddr::from(new_ip));
            self
        }

    pub fn set_port(&mut self, new_port: u16) -> &ConfigBuilder {
            self.addr.set_port(new_port);
            self
        }

}


#[cfg(test)]
mod tests {
    extern crate scopeguard;

    use super::*;

    #[test]
    fn builder_uses_default() {
        let def = Config::default();
        let built = ConfigBuilder::new().build();
        assert_eq!(def, built);
    }
    
    #[test]
    fn builder_sets_root() {
        let built = ConfigBuilder::new()
            .set_root(&PathBuf::from("rootdir"))
            .build();
        assert_eq!(PathBuf::from("rootdir"), built.rootdir);
    }

    #[test]
    fn builder_sets_longer_root() {
        let built = ConfigBuilder::new()
            .set_root("path/to/rootdir/")
            .build();
        assert_eq!(PathBuf::from("path/to/rootdir"), built.rootdir);
    }
    
    #[test]
    fn builder_sets_addr_tuple() {
        let addr_source = ([1,1,1,1], 8080);
        let built = Config::build()
            .set_address(addr_source)
            .build();
        assert_eq!(built.addr, SocketAddr::from(addr_source))
    }

    #[test]
    fn builder_sets_ip() {
        let addr_source = ([1,1,1,1], 8080);
        let built = Config::build()
            .set_ip(addr_source.0)
            .build();
        assert_eq!(built.addr.ip(), SocketAddr::from(addr_source).ip())
    }

    #[test]
    fn builder_sets_port() {
        let addr_source = ([1,1,1,1], 8080);
        let built = Config::build()
            .set_port(addr_source.1)
            .build();
        assert_eq!(built.addr.port(), SocketAddr::from(addr_source).port())
    }

    mod env_tests {
        use super::super::*;
        extern crate scopeguard;

        #[test]
        fn builder_sources_env_root() {
            let test_val = "test/root/dir";
            let _root_var = scopeguard::guard(env::var_os(ROOTDIR_KEY),
            |root_var| { 
                if let Some(root_var) = root_var {
                    env::set_var(ROOTDIR_KEY, root_var);
                } else {
                    env::remove_var(ROOTDIR_KEY);
                }
            });
            env::set_var(ROOTDIR_KEY, test_val);
            let c = ConfigBuilder::new()
                .source_env()
                .build();

            assert_eq!(PathBuf::from(test_val), c.rootdir);
        }

        #[test]
        fn builder_sources_env_static() {
            let test_val = "test/static/dir";
            let _static_var = scopeguard::guard(env::var_os(STATICDIR_KEY),
            |static_var| { 
                if let Some(static_var) = static_var {
                    env::set_var(STATICDIR_KEY, static_var);
                } else {
                    env::remove_var(STATICDIR_KEY);
                }
            });
            env::set_var(STATICDIR_KEY, test_val);
            let c = ConfigBuilder::new()
                .source_env()
                .build();

            assert_eq!(PathBuf::from(test_val), c.staticdir);
        }

        #[test]
        fn builder_sources_without_env() {
            let _root_var = scopeguard::guard(env::var_os(ROOTDIR_KEY),
            |root_var| { 
                if let Some(root_var) = root_var {
                    env::set_var(ROOTDIR_KEY, root_var);
                } else {
                    env::remove_var(ROOTDIR_KEY);
                }
            });
            let _static_var = scopeguard::guard(env::var_os(STATICDIR_KEY),
            |static_var| { 
                if let Some(static_var) = static_var {
                    env::set_var(STATICDIR_KEY, static_var);
                } else {
                    env::remove_var(STATICDIR_KEY);
                }
            });

            env::remove_var(ROOTDIR_KEY);
            env::remove_var(STATICDIR_KEY);
            let default_config = Config::default();
            let c = ConfigBuilder::new()
                .source_env()
                .build();

            assert_eq!(default_config.rootdir, c.rootdir);
            assert_eq!(default_config.staticdir, c.staticdir);
        }

    }
}
