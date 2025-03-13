use std::io::{stdin, BufRead};
use tour::Template;

#[derive(Template)]
#[template(root = "examples/dynamic/src/layout.html")]
struct Layout<'a, T: tour::Display> {
    body: T,
    path: &'a str,
}

#[derive(Template)]
#[template(root = "examples/dynamic/src/main.html")]
struct Page {
    name: String,
}

#[derive(Template)]
#[template(source = "<div>Inlined {{ name }}</div>")]
struct Inline {
    name: String,
}

fn main() {
    let path = "home";

    loop {
        let result = Layout {
            body: Page { name: "<script>alert('foo')</script>".into() },
            path,
        }.render().unwrap();

        let inlined = Inline { name: "foo".into() }.render().unwrap();

        println!("{result}{inlined}");
        println!("[Press ENTER to re render]");

        let mut buf = String::new();
        { stdin().lock().read_line(&mut buf).unwrap(); }

        if buf == "q\n" {
            break
        }
    }
}
