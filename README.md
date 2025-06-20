# Tour Template

A HTML templating library designed for Rust web applications.

## Compile-time templates

Templates are validated and optimized during compilation,
catching errors early and ensuring maximum performance.

## Runtime reloading (debug mode)

During development, templates are reloaded every render. (Enabled by
default in debug builds, disabled in release for efficiency.)

## Installation

To install `tour`, run:

```bash
cargo add tour
```

or add this line to your `Cargo.toml` in `[dependencies]` section:

```toml
tour = "0.1.0"
```

## Usage

```rust
// main.rs
use tour::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct Tasks {
    tasks: Vec<String>,
}

fn main() {
    let result = Tasks { tasks: vec![] }.render().unwrap();
    println!("{result}");
}
```

Templates are searched from `templates` directory from project root by default.

```html
<!-- templates/index.html -->
{{ for task in tasks }}
    Task: {{ task.get(1..6) }}
{{ else }}
    No Tasks
{{ endfor }}
```

In debug mode, changing non expression like `No Tasks` in the source file, will
change the output with the new content on the next render without recompiling.

Note that changing expression like `{{ for task in tasks }}` still requires recompile. An
attempt to render it without recompile, will change nothing and may result in error.

This is still better than require to recompile on every small changes. In practice, quick
changes iteration is used for style changes.

## License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

