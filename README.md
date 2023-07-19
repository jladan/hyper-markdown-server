# Hyper Markdown Server

A simple markdown server written in rust, using the fairly low-level
[hyper](https://crates.io/crates/hyper) library.

This project is a learning exercise to implement the same or better
functionality as my
[simple-markdown-server](https://github.com/jladan/simple-markdown-server), but
using an asynchronous runtime. It is still relatively low-level, using hyper
instead of another framework, because I want some practice in the nitty-gritty
parts of rust.
