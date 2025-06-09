//! The [`File`] struct.
use quote::format_ident;
use std::rc::Rc;
use syn::{Ident, LitStr, Result};

use crate::{
    ast::StmtTempl,
    data::Template,
    metadata::Metadata,
    syntax::{BlockTempl, LayoutTempl},
};

mod visitor;
mod validate;

use visitor::SynVisitor;
use validate::ValidateVisitor;

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
        let file = SynVisitor::generate(meta)?;
        ValidateVisitor::validate(&file)?;
        Ok(file)
    }

    /// Get block by id.
    pub fn get_block(&self, block: &Ident) -> Option<&BlockContent> {
        self.blocks.iter().find(|e| &e.templ.name == block)
    }

    pub(crate) fn block(&self, block: &Ident) -> &BlockContent {
        self.get_block(block).expect("[BUG] validation block rendering missed")
    }

    /// Get imported template by id.
    pub fn get_import_by_id(&self, name: &Ident) -> Option<&Import> {
        self.imports.iter().find(|&e| e == name)
    }

    pub(crate) fn import_by_id(&self, name: &Ident) -> &Import {
        self.get_import_by_id(name).expect("[BUG] validation import rendering missed")
    }

    /// Get imported template by path.
    pub fn get_import_by_path(&self, path: &LitStr) -> Option<&Import> {
        let path = path.value();
        self.imports.iter().find(|&e| e == &*path)
    }

    pub(crate) fn import_by_path(&self, path: &LitStr) -> &Import {
        self.get_import_by_path(path).expect("[BUG] validation import rendering missed")
    }

    pub(crate) fn resolve_id(&self, id: &Ident) -> AliasKind<'_> {
        match self.get_block(id) {
            Some(block) => AliasKind::Block(block),
            None => AliasKind::Import(self.import_by_id(id)),
        }
    }

    /// Returns all statements.
    pub fn stmts(&self) -> &[StmtTempl] {
        &self.stmts
    }

    /// Consume file into [`LayoutTempl`].
    pub fn into_layout(self) -> Option<LayoutTempl> {
        self.layout
    }

    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    pub fn blocks(&self) -> &[BlockContent] {
        &self.blocks
    }

    pub fn statics(&self) -> &[Rc<str>] {
        &self.statics
    }

    pub fn layout(&self) -> Option<&LayoutTempl> {
        self.layout.as_ref()
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
            None => gen_name_by_path(&"Import", self.path.as_ref()),
        }
    }
}

fn gen_name_by_path(prefix: &impl std::fmt::Display, path: &str) -> Ident {
    let suffix = std::path::Path::new(path)
        .file_stem()
        .and_then(|e|e.to_str())
        .unwrap_or("OsFile");
    format_ident!("{prefix}{suffix}")
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

