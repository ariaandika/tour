use tour::Template;

#[derive(Template)]
#[template(source = "
<section>
{{ pub block Body }}
  <p>Content</p>
{{ endblock }}
</section>
")]
struct Page;

#[derive(Template)]
#[template(source = r#"
{{ extends "/example/src/layout.html" }}
  <p>Content</p>
"#)]
struct PageLayouted;

#[derive(Template, Default)]
#[template(source = r#"
<form>
  <label>
    <span>Username</span>
    <input name="username">
  </label>
  <label>
    <span>Username</span>
    <input name="username">
  </label>

  {{ if let Some(err) = error }}
    <span id="error">{{ err }}</span>
  {{ endif }}

</form>
"#)]
struct Form {
    error: Option<String>,
}

fn main() {
    let page = Page;

    let _full_page = page.render().unwrap();
    let _inner_page = page.render_block("Body").unwrap();


    let form = Form::default();
    let _form = form.render().unwrap();
}
