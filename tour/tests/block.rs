use tour::Template;

#[test]
fn blocked() {
    #[derive(Template)]
    #[template(root = "tour/tests/block/block.html")]
    struct Root;

    let p = Root.render().unwrap();
    assert_eq!(
        &p[..],
        "<div><p>Willie</p></div>
<div><p>Jane</p></div>
<p>Willie</p>


");
}

#[test]
fn block_comptime() {
    #[derive(Template)]
    #[template(root = "tour/tests/block/block.html", block = Willie)]
    struct Block;

    let p = Block.render().unwrap();
    assert_eq!(&p[..], "<p>Willie</p>");
}

#[test]
fn block_runtime() {
    #[derive(Template)]
    #[template(root = "tour/tests/block/block.html")]
    struct Block;

    let p = Block.render_block("Body").unwrap();
    assert_eq!(&p[..], "<h1>Title</h1>");
}

