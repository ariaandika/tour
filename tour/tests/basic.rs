use tour::render_to;


#[test]
fn basic() {
    let mut buffer = String::new();
    render_to!(&mut buffer, {
        div id="false" {  }
    });
    assert_eq!(&buffer[..],"<div id=\"false\"></div>");
}

