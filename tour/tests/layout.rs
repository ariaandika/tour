use tour::Template;

#[test]
fn nested_layout() {
    #[derive(Template)]
    #[template(root = "tour/tests/layout/base.html")]
    struct NestedLayout {
        name: String,
    }

    let layout = NestedLayout { name: "barred".into() }.render_layout().unwrap();
    assert_eq!(
        &layout[..],
        r#"<section id="layout1"><section id="layout2"><section id="layout3">Token barred once
</section>
</section>
</section>
"#
    );
}

