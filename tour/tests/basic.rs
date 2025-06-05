use tour::Template;

struct MyDisplay;

impl tour::TemplDisplay for MyDisplay {
    fn display(&self, f: &mut impl tour::TemplWrite) -> tour::Result<()> {
        f.write_str("TemplDisplay trait")
    }
}

impl std::fmt::Display for MyDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Display trait")
    }
}

impl std::fmt::Debug for MyDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Debug trait")
    }
}

#[test]
fn render() {
    #[derive(Template)]
    #[template(source = "{{ input }}")]
    struct Displayed {
        input: MyDisplay,
    }

    let templ = Displayed { input: MyDisplay };

    assert_eq!(templ.render().unwrap(), "TemplDisplay trait");
}

#[test]
fn render_std_display_delimiter() {
    #[derive(Template)]
    #[template(source = "{% input %}")]
    struct Displayed {
        input: MyDisplay,
    }

    let templ = Displayed { input: MyDisplay };

    assert_eq!(templ.render().unwrap(), "Display trait");
}

#[test]
fn render_std_debug() {
    #[derive(Template)]
    #[template(source = "{? input ?}")]
    struct Displayed {
        input: MyDisplay,
    }

    let templ = Displayed { input: MyDisplay };

    assert_eq!(templ.render().unwrap(),"Debug trait");
}

#[test]
fn escape() {
    #[derive(Template)]
    #[template(source = "{{ input }}")]
    struct Escape {
        input: &'static str,
    }

    let templ = Escape { input: "<script></script>", };

    assert_eq!(templ.render().unwrap(), "&ltscript&gt&lt/script&gt");
}

#[test]
fn escape_opt_out() {
    #[derive(Template)]
    #[template(source = "{! input !}")]
    struct EscapeOptOut {
        input: &'static str,
    }

    let templ = EscapeOptOut { input: "<script></script>" };

    assert_eq!(templ.render().unwrap(), "<script></script>");
}

#[test]
fn conditional() {
    #[derive(Template)]
    #[template(source = "<div>{{ if *cond }}Ok{{ endif }}</div>")]
    struct Cond {
        cond: bool
    }

    let t1 = Cond { cond: false };
    assert_eq!(t1.render().unwrap(), "<div></div>");

    let t2 = Cond { cond: true };
    assert_eq!(t2.render().unwrap(), "<div>Ok</div>");

    let (min, Some(max)) = t1.size_hint() else { unreachable!() };

    assert_eq!(min, t1.render().unwrap().len());
    assert_eq!(max, t2.render().unwrap().len());
}

#[test]
fn conditional_2_branch() {
    #[derive(Template)]
    #[template(source = "<div>{{ if *cond }}Some{{ else }}None{{ endif }}</div>")]
    struct Cond {
        cond: bool
    }

    let t1 = Cond { cond: false };
    assert_eq!(t1.render().unwrap(), "<div>None</div>");

    let t2 = Cond { cond: true };
    assert_eq!(t2.render().unwrap(), "<div>Some</div>");

    let (min, Some(max)) = t1.size_hint() else { unreachable!() };

    assert_eq!(min, t1.render().unwrap().len());
    assert_eq!(max, t2.render().unwrap().len());
}

#[test]
fn conditional_3_branch() {
    #[derive(Template)]
    #[template(source = "{{ if *n == 1 }}One{{ else if *n == 2 }}Two{{ else }}None{{ endif }}")]
    struct Cond {
        n: i32,
    }

    let t1 = Cond { n: 1 };
    assert_eq!(t1.render().unwrap(), "One");

    let t2 = Cond { n: 2 };
    assert_eq!(t2.render().unwrap(), "Two");

    let t3 = Cond { n: 3 };
    assert_eq!(t3.render().unwrap(), "None");

    let (min, Some(max)) = t1.size_hint() else { unreachable!() };

    assert_eq!(min, t1.render().unwrap().len());
    assert_eq!(max, t3.render().unwrap().len());
}

#[test]
fn iteration() {
    #[derive(Template)]
    #[template(source = "<div>{{ for i in it }}the: {{ i }} {{ endfor }}</div>")]
    struct It {
        it: Vec<&'static str>,
    }

    let templ = It { it: vec!["A","B","C"] };
    assert_eq!(templ.render().unwrap(),"<div>the: A the: B the: C </div>");

    //
    // currently, size_hint will only guess max size_hint for one item iteration
    //

    let templ = It { it: vec!["A"] };
    let (min, Some(max)) = templ.size_hint() else { unreachable!() };

    let dynamic_size = "A".len();

    assert_eq!(min, "<div></div>".len());
    assert_eq!(max, templ.render().unwrap().len() - dynamic_size);
}

#[test]
fn iteration_2_branch() {
    #[derive(Template)]
    #[template(source = "<div>{{ for i in it }}the: {{ i }} {{ else }}None{{ endfor }}</div>")]
    struct It {
        it: Vec<&'static str>,
    }

    let t1 = It { it: vec!["A","B","C"] };
    assert_eq!(t1.render().unwrap(), "<div>the: A the: B the: C </div>");

    let t2 = It { it: vec![] };
    assert_eq!(t2.render().unwrap(), "<div>None</div>");

    let (min, Some(_max)) = t1.size_hint() else { unreachable!() };

    assert_eq!(min, t2.render().unwrap().len());
}

#[test]
fn using() {
    #[derive(Template)]
    #[template(source = "{{ use std::iter::once }}{? once(1).next() ?}")]
    struct Using;

    let templ = Using;

    assert_eq!(templ.render().unwrap(),"Some(1)");
}

