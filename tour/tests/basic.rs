use tour::Template;

struct MyDisplay;

impl tour::TemplDisplay for MyDisplay {
    fn display(&self, f: &mut impl tour::TemplWrite) -> tour::Result<()> {
        f.write_str("kernel")
    }
}

impl std::fmt::Display for MyDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("beans")
    }
}

impl std::fmt::Debug for MyDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ultra beans")
    }
}

#[test]
fn render() {
    #[derive(Template)]
    #[template(source = "{{ input }}")]
    struct Displayed {
        input: MyDisplay,
    }

    let esc = Displayed { input: MyDisplay }.render().unwrap();
    assert_eq!(&esc[..],"kernel");
}

#[test]
fn render_std_display_attr() {
    #[derive(Template)]
    #[template(source = "{{ input }}")]
    struct Displayed {
        #[fmt(display)]
        input: MyDisplay,
    }

    let esc = Displayed { input: MyDisplay }.render().unwrap();
    assert_eq!(&esc[..],"beans");
}

#[test]
fn render_std_display_delimiter() {
    #[derive(Template)]
    #[template(source = "{% input %}")]
    struct Displayed {
        input: MyDisplay,
    }

    let esc = Displayed { input: MyDisplay }.render().unwrap();
    assert_eq!(&esc[..],"beans");
}

#[test]
fn render_std_debug() {
    #[derive(Template)]
    #[template(source = "{? input ?}")]
    struct Displayed {
        input: MyDisplay,
    }

    let esc = Displayed { input: MyDisplay }.render().unwrap();
    assert_eq!(&esc[..],"ultra beans");
}

#[test]
fn escape() {
    #[derive(Template)]
    #[template(source = "{{ input }}")]
    struct Escape {
        input: &'static str,
    }

    let esc = Escape { input: "<script></script>" }.render().unwrap();
    assert_eq!(&esc[..],"&ltscript&gt&lt/script&gt");
}

#[test]
fn escape_opt_out() {
    #[derive(Template)]
    #[template(source = "{! input !}")]
    struct EscapeOptOut {
        input: &'static str,
    }

    let esc = EscapeOptOut { input: "<script></script>" }.render().unwrap();
    assert_eq!(&esc[..],"<script></script>");
}

#[test]
fn conditional() {
    #[derive(Template)]
    #[template(source = "<div>{{ if *cond }}Ok{{ endif }}</div>")]
    struct Cond {
        cond: bool
    }

    let cond = Cond { cond: true }.render().unwrap();
    assert_eq!(&cond[..],"<div>Ok</div>");

    let cond = Cond { cond: false }.render().unwrap();
    assert_eq!(&cond[..],"<div></div>");
}

#[test]
fn conditional_2_branch() {
    #[derive(Template)]
    #[template(source = "<div>{{ if *cond }}Some{{ else }}None{{ endif }}</div>")]
    struct Cond {
        cond: bool
    }

    let cond = Cond { cond: true }.render().unwrap();
    assert_eq!(&cond[..],"<div>Some</div>");

    let cond = Cond { cond: false }.render().unwrap();
    assert_eq!(&cond[..],"<div>None</div>");
}

#[test]
fn conditional_3_branch() {
    #[derive(Template)]
    #[template(source =
        "<div>{{ if *n == 1 }}One{{ else if *n == 2 }}Two{{ else }}None{{ endif }}</div>"
    )]
    struct Cond {
        n: i32
    }

    let cond = Cond { n: 1 }.render().unwrap();
    assert_eq!(&cond[..],"<div>One</div>");

    let cond = Cond { n: 2 }.render().unwrap();
    assert_eq!(&cond[..],"<div>Two</div>");

    let cond = Cond { n: 3 }.render().unwrap();
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
}

#[test]
fn iteration_2_branch() {
    #[derive(Template)]
    #[template(source = "<div>{{ for i in it }}the: {{ i }} {{ else }}None{{ endfor }}</div>")]
    struct It {
        it: Vec<&'static str>,
    }

    let it = It { it: vec!["A","B","C"] }.render().unwrap();
    assert_eq!(&it[..],"<div>the: A the: B the: C </div>");

    let it = It { it: vec![] }.render().unwrap();
    assert_eq!(&it[..],"<div>None</div>");
}

#[test]
fn using() {
    #[derive(Template)]
    #[template(source = "{{ use std::io::{Read, Write, prelude::*} }}")]
    struct Using;

    let u = Using.render().unwrap();
    assert_eq!(&u[..],"");
}

