use quote::format_ident;
use std::rc::Rc;
use syn::{Ident, Result};

use crate::{
    ast::StmtTempl,
    data::Template,
    metadata::Metadata,
    syntax::{BlockTempl, LayoutTempl},
};

mod visitor;

use visitor::SynVisitor;

/// Content of a template source.
pub struct File {
    layout: Option<LayoutTempl>,
    imports: Vec<Import>,
    blocks: Vec<BlockContent>,
    statics: Vec<Rc<str>>,
    stmts: Vec<StmtTempl>,
}

pub struct BlockContent {
    pub templ: BlockTempl,
    pub stmts: Vec<StmtTempl>,
}

impl File {
    /// Create [`File`] from [`Metadata`].
    pub fn from_meta(meta: &Metadata) -> Result<File> {
        SynVisitor::generate(meta)
    }

    /// Returns all statements.
    pub fn stmts(&self) -> &[StmtTempl] {
        &self.stmts
    }

    /// Consume file into [`LayoutTempl`].
    pub fn into_layout(self) -> Option<LayoutTempl> {
        self.layout
    }
}

// ===== Import =====

pub struct Import {
    path: Rc<str>,
    alias: Option<Ident>,
    templ: Template,
}

impl Import {
    pub(crate) fn new(path: Rc<str>, alias: Option<Ident>, templ: Template) -> Self {
        Self { path, alias, templ }
    }

    pub fn templ(&self) -> &Template {
        &self.templ
    }

    pub fn generate_name(&self) -> Ident {
        match &self.alias {
            Some(name) => format_ident!("Import{name}"),
            None => {
                let suffix = std::path::Path::new(&*self.path)
                    .file_stem()
                    .and_then(|e|e.to_str())
                    .unwrap_or("OsFile");
                format_ident!("Import{suffix}")
            },
        }
    }
}

impl PartialEq<str> for Import {
    fn eq(&self, other: &str) -> bool {
        self.path.as_ref() == other
    }
}

impl PartialEq<Ident> for Import {
    fn eq(&self, other: &Ident) -> bool {
        match self.alias.as_ref() {
            Some(id) => id == other,
            None => false,
        }
    }
}

// ===== AliasKind =====

pub enum AliasKind<'a> {
    Block(&'a BlockContent),
    Import(&'a Import),
}

