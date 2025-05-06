use std::io::{stdin, BufRead};
use tour::Template;

#[derive(Template)]
#[template(root = "example/src/main.html")]
struct Page {
    id: i32,
    name: String,
}

#[derive(Template)]
#[template(source = "<div>Inlined {{ name }}</div>")]
struct Inline {
    name: String,
}

fn main() {
    let _path = "home";

    loop {
        let page = Page {
            id: 4,
            name: "<script>alert('foo')</script>".into()
        };
        let result = page.render_layout().unwrap();

        let inlined = Inline { name: format!("foo {}",4) }.render().unwrap();

        println!("{result}{inlined}");
        println!("[Press ENTER to re render]");

        let mut buf = String::new();
        { stdin().lock().read_line(&mut buf).unwrap(); }

        if buf == "q\n" {
            break
        }
    }
}
