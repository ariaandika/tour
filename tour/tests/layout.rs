use tour::Template;

#[test]
fn nested_layout() {
    #[derive(Template)]
    #[template(root = "tour/tests/layout/base.html")]
    struct NestedLayout {
        name: String,
    }

    let layout = NestedLayout { name: "barred".into() }.render().unwrap();
    assert_eq!(
        &layout[..],
        r#"<section id="layout3"><section id="layout2"><section id="layout1">Token barred once
</section>
</section>
</section>
"#
    );
}

