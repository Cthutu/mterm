# mterm

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/cthutu/mterm/Rust?logo=GitHub&style=flat-square)
![Crates.io](https://img.shields.io/crates/d/mterm?logo=Rust&style=flat-square)
![Crates.io](https://img.shields.io/crates/l/mterm?logo=Rust&style=flat-square)
![Crates.io](https://img.shields.io/crates/v/mterm?logo=Rust&style=flat-square)
![GitHub repo size](https://img.shields.io/github/repo-size/cthutu/mterm?logo=Github&style=flat-square)

This crate provides a framework for implementing an application that requires a single window with ASCII text.
It uses the GPU to render the ASCII quickly and provides a trait of two methods so that user's code can hook into
the main loop that is implemented by the crate.

# Examples

There is one example that demonstrates a simple application that implements `mterm::App` which constructs a `mterm::Image` 
containing the "Hello World" message, and blits it on to the screen.

