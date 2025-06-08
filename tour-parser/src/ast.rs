//! Full syntax definition including nested scope for template statements.
use std::rc::Rc;
use syn::*;
use tour_core::Delimiter;

use crate::syntax::*;

/// Template statement.
pub enum StmtTempl {
    /// Regular statement.
    Scalar(Scalar),
    /// Scoped statement.
    Scope(Scope),
}

pub enum Scalar {
    /// Static content.
    Static {
        value: Rc<str>,
        index: u32,
    },
    /// Rust expression.
    Expr(Rc<Expr>,Delimiter),
    /// Render block or external template.
    Render(RenderTempl),
    /// Import and alias external template.
    Use(UseTempl),
    /// Render body for layout.
    Yield,
}

/// Scoped rust statement.
pub enum Scope {
    /// Block of statements.
    Root { stmts: Vec<StmtTempl> },
    /// If statement.
    If {
        templ: IfTempl,
        stmts: Vec<StmtTempl>,
        else_branch: Option<(Token![else],Box<Scope>)>
    },
    /// For statement.
    For {
        templ: ForTempl,
        stmts: Vec<StmtTempl>,
        else_branch: Option<(Token![else],Box<Scope>)>
    },
    /// Block declaration.
    Block {
        templ: BlockTempl,
        stmts: Vec<StmtTempl>,
    },
}

impl Scope {
    pub(crate) fn stack_mut(&mut self) -> &mut Vec<StmtTempl> {
        match self {
            Self::Root { stmts } => stmts,
            Self::Block { stmts, .. } => stmts,
            Self::For { else_branch: Some(branch), .. } => branch.1.stack_mut(),
            Self::For { stmts, .. } => stmts,
            Self::If { else_branch: Some(branch), .. } => branch.1.stack_mut(),
            Self::If { stmts, .. } => stmts,
        }
    }
}

