pub mod context;
pub mod config;
pub mod uri;
pub mod response;
pub mod handler;


use std::{
    path::{Path, PathBuf, StripPrefixError},
    ffi::{OsStr, OsString},
};

use walkdir::{WalkDir, DirEntry};

use serde::Serialize;


/* Using Separate structs for files and directories makes it much easier to build from WalkDir
 * In addition, it forces my to store separate lists of subdirectories and files, which means the
 * values are already sorted by type.
 */
#[derive(Debug, Clone, Serialize)]
pub struct Directory { 
    name: String, 
    // NOTE(jladan): A path is best for creating the tree, but if it is used for links, this will
    // need a `/` prepended to it.
    path: PathBuf, 
    dirs: Vec<Directory>,
    files: Vec<File>,
}

#[derive(Debug, Clone, Serialize)]
pub struct File { 
    name: String, 
    path: String,
    media_type: Option<String>,
}

impl Directory {
    fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
            dirs: Vec::new(),
            files: Vec::new(),
        }
    }
}

impl File {
    fn new(name: &OsStr, path: OsString) -> Self {
        Self { 
            name: name.to_string_lossy().to_string(), 
            path: path.to_string_lossy().to_string(),
            media_type: None
        }
    }
}


/*
FS traversal
------------

Walkdir is just a depth first iterator through the fs tree. The entire path is shown during the
walk, so tree-structure can be determined through the path prefix.
*/
/// Walks the given path, returning a serializable directory tree
/// 
/// Uses the `walkdir` crate to iterate over all files at the given path. Returns a [Directory],
/// which contains `vecs` of all subdirectories and files.
///
/// ## Arguments:
/// - `path: &Path`: the path to the directory
/// - `absolute: bool` -- whether the returned paths should start with "/"
///
pub fn walk_dir(path: &Path, absolute: bool) -> Result<Directory, StripPrefixError> {
    // Prefix to strip from all paths
    let prefix = path;
    // Stack for depth-first search
    let mut dirstack: Vec<Directory> = Vec::new();
    // Set up walkdir iterator, sorted by filename with no hidden files
    let mut walker = WalkDir::new(prefix)
        .sort_by(|a,b| a.file_name().to_ascii_lowercase().cmp(&b.file_name().to_ascii_lowercase()))
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok());
    let mut curdir: Directory;
    if let Some(entry) = walker.next() {
        let stripped = entry.path().strip_prefix(prefix)?;
        curdir = Directory::new("/", stripped)
    } else {
        // TODO(jladan): This should only happen if the requested path is unreachable, which should
        // actually be an error
        curdir = Directory::new("/", &PathBuf::from(""))
    };
    for entry in walker {
        // XXX(jladan): because we start at `prefix`, this error should never happen
        // - would following a link break this?
        let stripped = entry.path().strip_prefix(prefix)
            .expect("Walking through the directory left prefix");
        if !stripped.starts_with(&curdir.path) {
            // Left the current directory
            // Need to find parent in the stack
            while let Some(mut prevdir) = dirstack.pop() {
                // Add the current directory to its parent
                format_dir(&mut curdir.path);
                if absolute { *curdir.path.as_mut_os_string() = make_abs(&curdir.path) }
                prevdir.dirs.push(curdir);
                curdir = prevdir;
                // Continue until we've found the parent
                // XXX: curdir.path has been made absolute... A problem?
                if stripped.starts_with(&curdir.path) {
                    break;
                }
            }
        }
        // Now perform logic on current entry
        if entry.file_type().is_file() {
            let stripped = if absolute { 
                make_abs(stripped)
            } else {
                stripped.as_os_str().to_os_string()
            };
            curdir.files.push(File::new(entry.file_name(), stripped));
        } else if entry.file_type().is_dir() {
            // Push current directory to stack, and start processing next one
            // NOTE(jladan): new directory still needs to be added to "current"
            dirstack.push(curdir);
            curdir = Directory::new(&entry.file_name().to_string_lossy(), stripped);
        }
    }
    // Now, unstack all the way to root
    while let Some(mut prevdir) = dirstack.pop() {
        // Add the current directory to its parent
        format_dir(&mut curdir.path);
        if absolute { *curdir.path.as_mut_os_string() = make_abs(&curdir.path) }
        prevdir.dirs.push(curdir);
        curdir = prevdir;
    }
    if absolute { *curdir.path.as_mut_os_string() = make_abs(&curdir.path) }
    return Ok(curdir)
}

/// Adds a trailing slash to all directories
fn format_dir(a: &mut PathBuf) {
    a.as_mut_os_string().push("/");
}

/// Prepends a `/` to directories so the path is "absolute"
fn make_abs(a: &Path) -> OsString {
    let mut built = OsString::with_capacity(a.as_os_str().len() + 1);
    built.push("/");
    built.push(a.as_os_str());
    built
}

/// Checks if a file starts with "."
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
