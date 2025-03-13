# Tour

HTML Templating in rust

## Another ? What`s the catch ?

A compiled template where parts of it can be reloaded at runtime in debug mode.

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
```html
<!-- templates/index.html -->
{{ for task in tasks }}
    full rust syntax here {{ task.get(1..6) }}
{{ else }}
    No Tasks
{{ endfor }}
```

in debug mode, changing non expression like `No Tasks` in the source file,
the next render will output the new content

