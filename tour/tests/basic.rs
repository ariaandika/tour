use tour::render_to;


#[test]
fn basic() {
    let mut buffer = String::new();
    let title = "tasks";
    let tasks = vec![
        ("foo","listed",true),
        ("bar","listed",true),
        ("baz","listed",true),
        ("bin","draft",false),
    ];

    render_to!{&mut buffer, {
        let _ = title;
        let Some(_) = title.get(0..4) else {
            return;
        };

        #title;
        println!("Oof");

        if let Some(_) = title.get(0..2) {

        }

        for _ in title.chars() {
            div id={title};
        }

    }}

    // h1 id={title} # {title}
    //
    // for task in &tasks {
    //     if !task.2 {
    //         continue;
    //     }
    //
    //     div # {task.0}
    // }

    assert_eq!(
        &buffer[..],
        "\
        <h1 id=\"tasks\">tasks</h1>\
        <div>foo</div>\
        <div>bar</div>\
        <div>baz</div>\
        "
    );
}

