//! The [`File`] struct.
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

    fn import_by_id(&self, name: &Ident) -> &Import {
        self.get_import_by_id(name)
            .unwrap_or_else(|| panic!("[BUG] validation import id missed, cannot find `{name}`: {:#?}",self.imports()))
    }

    /// Get imported template by path.
    pub fn get_import_by_path(&self, path: &LitStr) -> Option<&Import> {
        let path = path.value();
        self.imports.iter().find(|&e| e == &*path)
    }

    pub(crate) fn import_by_path(&self, path: &LitStr) -> &Import {
        self.get_import_by_path(path).unwrap_or_else(|| {
            panic!(
                "[BUG] validation import path missed, cannot find `{}`",
                path.value()
            )
        })
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

    /// Returns all mutable statements.
    pub fn stmts_mut(&mut self) -> &mut Vec<StmtTempl> {
        &mut self.stmts
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

    pub fn blocks_mut(&mut self) -> &mut Vec<BlockContent> {
        &mut self.blocks
    }

    pub fn statics(&self) -> &[Rc<str>] {
        &self.statics
    }

    pub fn layout(&self) -> Option<&LayoutTempl> {
        self.layout.as_ref()
    }
}

// ===== Import =====

#[derive(Debug)]
pub struct Import {
    path: Rc<str>,
    alias: Ident,
    templ: Template,
}

impl Import {
    pub(crate) fn new(path: Rc<str>, alias: Ident, templ: Template) -> Self {
        Self { path, alias, templ }
    }

    pub fn templ(&self) -> &Template {
        &self.templ
    }

    pub fn alias(&self) -> &Ident {
        &self.alias
    }
}

impl PartialEq<str> for Import {
    fn eq(&self, other: &str) -> bool {
        self.path.as_ref() == other
    }
}

impl PartialEq<Ident> for Import {
    fn eq(&self, other: &Ident) -> bool {
        &self.alias == other
    }
}

// ===== AliasKind =====

pub enum AliasKind<'a> {
    Block(&'a BlockContent),
    Import(&'a Import),
}

