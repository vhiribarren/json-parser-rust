# Simple JSON parser in Rust

This is a toy project to:

- train myself in using Rust
- train myself in coding parsers

No specific optimization, and not designed to be highly performant.
Currently, it only does parsing, and not serialization.

On the plus side:
- no usage of libraries outside of the `std` one
- no usage of a regular expression library

To build:

    $ cargo build

Some examples:

    $ cargo test
    $ cargo run --example json-debug -- -s '{"one": 1, "two": {"table":[1, null, true, {"bloup": 3}]}}'