use crate::Renderer;


pub trait Template {
    fn render_into(&self, render: &mut impl Renderer) -> Result<()>;

    fn render(&self) -> Result<String> {
        let mut buffer = String::with_capacity(1024);
        self.render_into(&mut buffer)?;
        Ok(buffer)
    }
}

pub type Result<T,E = Error> = std::result::Result<T,E>;

pub enum Error {

}

