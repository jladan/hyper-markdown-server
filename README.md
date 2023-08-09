# Hyper Markdown Server

A simple markdown server written in rust, using the fairly low-level
[hyper](https://crates.io/crates/hyper) library.

This project is a learning exercise to implement the same or better
functionality as my
[simple-markdown-server](https://github.com/jladan/simple-markdown-server), but
using an asynchronous runtime. It is still relatively low-level, using hyper
instead of another framework, because I want some practice in the nitty-gritty
parts of rust.

## TODO

- [ ] Figure out why partials for svg don't work
- [ ] Ability to create, edit, move, or delete from browser?
    - [ ] POST method to resources for moving them
        - [ ] reply with a redirect
    - [ ] DELETE to delete files
    - [ ] PUT method could be used to create the file
        - [ ] Open up in editor to actually make changes
    - [ ] PUT or PATCH can be used to make changes to a file.
        - this would require an in-browser editor :(
        - a PUT operation would be safer, because it is idempotent

