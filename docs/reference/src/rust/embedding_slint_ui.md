# How to use a Slint UI from rust

Designs of user interfaces are described in the `.slint` design markup language. There are three ways
of including them in Rust:

 - The `.slint` code is [inline in a macro](#the-slint-code-in-a-macro).
 - The `.slint` code in [external files compiled with `build.rs`](#the-slint-code-in-external-files-is-compiled-with-buildrs)
 - The `.slint` code is loaded dynamically at run-time from the file system, by using the [interpreter API](https://slint.dev/releases/", env!("CARGO_PKG_VERSION"), "/docs/rust/slint_interpreter/").

With the first two methods, the markup code is translated to Rust code and each component is turned into a Rust
struct with functions. Use these functions to instantiate and show the component, and
to access declared properties. Check out our [sample component](docs::generated_code::SampleComponent) for more
information about the generation functions and how to use them.

## The .slint code in a macro

This method combines your Rust code with the `.slint` design markup in one file, using a macro:

```rust
slint::slint!{
    export component HelloWorld {
        Text {
            text: "hello world";
            color: green;
        }
    }
}

fn main() {
#   return; // Don't run a window in an example
    HelloWorld::new().unwrap().run().unwrap();
}
```

## The .slint code in external files is compiled with `build.rs`

When your design becomes bigger in terms of markup code, you may want move it to a dedicated
`.slint` file. It's also possible to split a `.slint` file into multiple files using [modules](https://slint.dev/releases/", env!("CARGO_PKG_VERSION"), "/docs/slint/src/language/syntax/modules.html).")]
Use a [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) to compile
your main `.slint` file:

In your Cargo.toml add a `build` assignment and use the `slint-build` crate in `build-dependencies`:

```toml
[package]
...
build = "build.rs"
edition = "2021"

[dependencies]
slint = "{{release}}"
...

[build-dependencies]
slint-build = "{{release}}"
```

Use the API of the slint-build crate in the `build.rs` file:

```rust,no_run
fn main() {
    slint_build::compile("ui/hello.slint").unwrap();
}
```

Finally, use the [`include_modules!`] macro in your `main.rs`:

```ignore
slint::include_modules!();
fn main() {
    HelloWorld::new().unwrap().run().unwrap();
}
```
