//! partially compiled template
//!
//! in debug mode, static content is read from the file every render
//!
//! in release mode, its embedded
//!
//! logic template is not reloaded, recompile is required
use parser::Parser;

mod token;
mod parser;

const SOURCE: &str = "
Token {{ data }} app123

{{ if let Some(foo) = close }}
    Result me
{{ else if ok }}
    Anyhow
{{ else }}
    Nothing
{{ endif }}

{{ for i in tasks }}
    Clippy
{{ endfor }}

glippy
";

const _OUTPUT_FORMATTED: &str = r#"
writer.write_str("\nToken ")?;
write!(writer, "{}", data)?;
writer.write_str(" app123\n\n")?;
if let Some(foo) = close {
    writer.write_str("\n    Result me\n")?;
} else if ok {
    writer.write_str("\n    Anyhow\n")?;
} else {
    writer.write_str("\n    Nothing\n")?;
}
writer.write_str("\n\n")?;
for i in tasks {
    writer.write_str("\n    Clippy\n")?;
}
writer.write_str("\n\nglippy\n")?;
"#;

fn main() {
    let template = Parser::new(SOURCE).parse().unwrap();
    let stmts = template.stmts();
    let tokens = quote::quote! {#(#stmts)*};
    println!("fn main() {{ {tokens} }}");
}
