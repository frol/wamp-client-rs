# WAMP-proto client library for Rust

It is a new implemention of WAMP-proto specification in Rust using the new
[async-await Rust
syntax](https://blog.rust-lang.org/2019/11/07/Async-await-stable.html) and based
on [tokio 0.2-alpha](https://tokio.rs/blog/2019-08-alphas/). The current aim of
this crate is to only implement the client roles: caller, callee, publisher, and
subscriber.

There is some early experiment with the protocol in the `experiment` branch.
Keep in mind that all of that is just ad-hoc experiments, which will shape into
Rust API design *later*. I only publish those to keep track of different
experiments of my own. As of the day of this writing, caller and callee roles
barely work (I can call the callee functions and receive a response via crossbar
and nexus WAMP-proto routers, however, the arguments and the return values are
currently ignored and the error handling is completely missing!)
