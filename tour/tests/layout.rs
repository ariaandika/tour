use tour::Template;

#[test]
fn nested_layout() {
    #[derive(Template)]
    #[template(root = "tour/tests/layout/base.html")]
    struct NestedLayout {
        name: String,
    }

    impl NestedLayout {
        fn title(&self) -> &'static str {
            "Main Title"
        }
    }

    let layout = NestedLayout { name: "barred".into() }.render().unwrap();
    assert_eq!(
        &layout[..],
        r#"<section id="layout3" title="Main Title"><section id="layout2"><section id="layout1">Token barred once
</section>
</section>
</section>
"#
    );
}

