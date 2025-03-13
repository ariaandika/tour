use tour::Template;



#[derive(Template)]
#[template(root = "tour/tests/basic.html")]
struct Page {
    name: String,
}

#[test]
fn basic() {
    let ok = Page { name: "barred".into() }.render().unwrap();
    assert_eq!(&ok[..],"Token barred once\n");
}

