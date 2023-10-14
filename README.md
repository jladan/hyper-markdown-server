# Hyper Markdown Server

A simple markdown file server written in rust, using the fairly low-level
[hyper](https://crates.io/crates/hyper) library. The intended use is to view
files in my notes directory.

The pages are generated using templates, providing 3 types of responses
1. Directories: provide a list of files
2. Markdown files: 
    a. As a normal request: provides the rendered markdown, and a full filesystem tree
    b. With `x-partial: true` header: just responds with the rendered markdown
3. Other files (e.g. images, pdfs): 
    a. Normal request: sends the file
    b. With `x-partial` header: wraps the file in an appropriate html tag.

Using the custom `x-partial` header allows for using the [Location
API](https://developer.mozilla.org/en-US/docs/Web/API/Location) to update the
previewed content without re-sending the whole directory navigator, or losing it
in the case of images and pdfs.

This project is a learning exercise to implement the same or better
functionality as my
[simple-markdown-server](https://github.com/jladan/simple-markdown-server), but
using an asynchronous runtime. It is still relatively low-level, using hyper
instead of another framework, because I want some practice in the nitty-gritty
parts of rust. Using Hyper rather than Axum, Actix or other frameworks also
simplifies creating my own routes to the filesystem, rather than adding
middleware and adding an additional routing layer to find and process the file.

The downside of not using a higher-level framework is that I don't get easy
access to middleware like authorization or tracing. However, this is intended to
run locally with only get requests, so that's fine for my purposes.

## TODO

- [ ] Figure out why partials for svg don't work
- [ ] MAYBE: Ability to create, edit, move, or delete from browser?
    - [ ] POST method to resources for moving them
        - [ ] reply with a redirect
    - [ ] DELETE to delete files
    - [ ] PUT method could be used to create the file
        - [ ] Open up in editor to actually make changes
    - [ ] PUT or PATCH can be used to make changes to a file.
        - this would require an in-browser editor :(
        - a PUT operation would be safer, because it is idempotent

