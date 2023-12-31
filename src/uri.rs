//! URI lookup library
//!
//! This library provides the `resolve()` function, which will map the URI from a get request to a
//! local file.
//!

use std::{
    path::{PathBuf, Path}, ffi::OsStr,
};

use url_escape::decode as decode_url;

use crate::config::Config;

#[derive(Debug)]
pub enum Resolved {
    File(PathBuf),
    Markdown(PathBuf),
    Directory(PathBuf),
}

impl AsRef<Path> for Resolved {
    fn as_ref(&self) -> &Path {
        match self {
            Self::File(p) => p.as_ref(),
            Self::Markdown(p) => p.as_ref(),
            Self::Directory(p) => p.as_ref(),
        }
    }
}

pub fn resolve(uri: &hyper::Uri, config: &Config) -> Option<Resolved> {
    eprintln!("{:?}", uri.path());
    let relpath = force_relative(&decode_url(uri.path()));
    eprintln!("{:?}", relpath);
    let path = config.rootdir.join(&relpath);

    // TODO: support markdown files without an extension?
    //       what if there is both a file and directory: `things.md`, `things/stuff.md` ?
    if path.is_dir() {
        return Some(Resolved::Directory(path));
    } else if path.is_file() {
        return if path.extension() == Some(&OsStr::new("md")) {
            Some(Resolved::Markdown(path))
        } else {
            Some(Resolved::File(path))
        }
    }
    // }}} 
    // Look in the staticdir
    let path = config.staticdir.join(&relpath);
    if path.is_file() {
        return Some(Resolved::File(path));
    }
    // Nothing found
    None
}

fn force_relative(uri: &str) -> PathBuf {
    assert!(uri.starts_with('/'), 
            "The uri path for a request should always be absolute");
    PathBuf::from(&uri[1..])
}
