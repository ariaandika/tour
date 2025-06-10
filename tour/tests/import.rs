use tour::Template;

#[test]
fn import() {
    #[derive(Template)]
    #[template(path = "/tour/tests/import/import.html")]
    struct Import;

    let templ = Import;
    assert_eq!(
        templ.render().unwrap(),
        "<nav>Navbar</nav>\n\n\n<nav>Navbar</nav>\n\n\n"
    );
}

#[test]
fn import_alias() {
    #[derive(Template)]
    #[template(path = "/tour/tests/import/alias.html")]
    struct Import;

    let templ = Import;
    assert_eq!(
        templ.render().unwrap(),
        "\n<nav>Navbar</nav>\n\n\n<nav>Navbar</nav>\n\n\n"
    );
}

#[test]
fn import_block() {
    #[derive(Template)]
    #[template(path = "/tour/tests/import/block.html")]
    struct Import;

    let templ = Import;
    assert_eq!(
        templ.render().unwrap(),
        "\n\n<p>Title Block</p>\n\n\n<p>Title Block</p>\n\n"
    );
}
