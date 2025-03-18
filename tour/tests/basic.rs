use tour::Template;

#[test]
fn nested_layout() {
    #[derive(Template)]
    #[template(root = "tour/tests/basic.html")]
    struct NestedLayout {
        name: String,
    }

    let layout = NestedLayout { name: "barred".into() }.render_layout().unwrap();
    assert_eq!(&layout[..],r#"<section id="layout1">
  
<section id="layout2">
  
<section id="layout3">
  
Token barred once

</section>

</section>

</section>
"#);
}

#[test]
fn conditional() {
    #[derive(Template)]
    #[template(source = "<div>{{ if *cond }}Ok{{ endif }}</div>")]
    struct Cond1 {
        cond: bool
    }

    let cond = Cond1 { cond: true }.render().unwrap();
    assert_eq!(&cond[..],"<div>Ok</div>");

    let cond = Cond1 { cond: false }.render().unwrap();
    assert_eq!(&cond[..],"<div></div>");

    #[derive(Template)]
    #[template(source = "<div>{{ if *cond }}Some{{ else }}None{{ endif }}</div>")]
    struct Cond2 {
        cond: bool
    }

    let cond = Cond2 { cond: true }.render().unwrap();
    assert_eq!(&cond[..],"<div>Some</div>");

    let cond = Cond2 { cond: false }.render().unwrap();
    assert_eq!(&cond[..],"<div>None</div>");
}

#[test]
fn iteration() {
    #[derive(Template)]
    #[template(source = "<div>{{ for i in it }}the: {{ i }} {{ endfor }}</div>")]
    struct It {
        it: Vec<&'static str>,
    }

    let it = It { it: vec!["A","B","C"] }.render().unwrap();
    assert_eq!(&it[..],"<div>the: A the: B the: C </div>");

    #[derive(Template)]
    #[template(source = "<div>{{ for i in it }}the: {{ i }} {{ else }}None{{ endfor }}</div>")]
    struct It2 {
        it: Vec<&'static str>,
    }

    let it = It { it: vec!["A","B","C"] }.render().unwrap();
    assert_eq!(&it[..],"<div>the: A the: B the: C </div>");

    let it = It2 { it: vec![] }.render().unwrap();
    assert_eq!(&it[..],"<div>None</div>");
}

