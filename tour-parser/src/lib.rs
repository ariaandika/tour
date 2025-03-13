// kinds of layouting:
//
// - user create separate layout struct which have field to contains another struct
//   no special template feature required, very flexible and verbose
// - jinja like extends, using combination of `block` and `extends`, flexible and verbose
// - using `layout` directive, automatically create new type struct internally, simple but limited
//

pub mod parser;
pub mod token;

pub use parser::{Parser, Template};
