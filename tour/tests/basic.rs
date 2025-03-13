use tour::Template;



#[derive(Template)]
#[template(source = "Token {{ name }} once\n")]
struct InlinePage {
    name: String,
}

#[derive(Template)]
#[template(root = "tour/tests/basic.html")]
struct Page {
    name: String,
}

#[test]
fn basic() {
    let ok = Page { name: "barred".into() }.render().unwrap();
    assert_eq!(&ok[..],"Token barred once\n");
    let ok = InlinePage { name: "barred".into() }.render().unwrap();
    assert_eq!(&ok[..],"Token barred once\n");
}

