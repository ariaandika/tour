use tour::render_to;


#[test]
fn basic() {
    let mut buffer = String::new();

    let id = "app";

    render_to!(&mut buffer, {
        div id={id} {  }

        for i in id.chars() {
            div # i;
        }

    });

    assert_eq!(&buffer[..],"<div id=\"false\"></div>");
}

