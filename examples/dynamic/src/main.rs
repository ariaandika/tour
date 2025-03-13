use std::io::{stdin, BufRead};

use tour::Template;



#[derive(Template)]
#[template(root = "examples/dynamic/src/main.html")]
struct Page {
    name: String,
}

fn main() {
    loop {
        let result = Page { name: "foo".into() }.render().unwrap();
        println!("{}",result);

        println!("[Press ENTER to re render]");

        let mut buf = String::new();
        { stdin().lock().read_line(&mut buf).unwrap(); }

        if buf == "q\n" {
            break
        }
    }
}
