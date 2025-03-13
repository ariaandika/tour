use std::io::{stdin, BufRead};

use tour::Template;


#[derive(Template)]
#[template(root = "examples/dynamic/src/layout.html")]
struct Layout<T: tour::Render> {
    body: T,
}

#[derive(Template)]
#[template(root = "examples/dynamic/src/main.html")]
struct Page {
    name: String,
}

fn main() {
    loop {
        let result = Layout {
            body: Page { name: "<script>alert('foo')</script>".into() }
        }.render().unwrap();

        println!("{}",result);
        println!("[Press ENTER to re render]");

        let mut buf = String::new();
        { stdin().lock().read_line(&mut buf).unwrap(); }

        if buf == "q\n" {
            break
        }
    }
}
