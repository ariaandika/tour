use syn::*;
use tour_core::{ParseError, Parser};

use crate::{
    attribute::AttrData,
    shared::{Reload, Source, error},
    syntax::{BlockTempl, LayoutTempl},
    visitor::{StmtTempl, SynVisitor},
};

/// Extra information declared outside template file.
pub struct Metadata {
    path: Option<Box<str>>,
    reload: Reload,
    block: Option<Ident>,
}

impl Metadata {
    pub fn from_attr(attr: &AttrData) -> Metadata {
        Self {
            path: attr.source().clone_path(),
            reload: attr.reload().clone(),
            block: attr.block().cloned(),
        }
    }

    pub fn from_layout(layout: &LayoutTempl, reload: Reload, cwd: Option<Box<str>>) -> Result<Metadata> {
        let source = Source::from_layout(layout, cwd)?;
        Ok(Self {
            path: source.clone_path(),
            reload,
            block: None, // allows select block for a layout ?
        })
    }
}

/// Content of a template file.
pub struct File {
    layout: Option<LayoutTempl>,
    blocks: Vec<BlockContent>,
    statics: Vec<Box<str>>,
    stmts: Vec<StmtTempl>,
}

impl File {
    /// Create new [`File`].
    pub fn new(
        layout: Option<LayoutTempl>,
        blocks: Vec<BlockContent>,
        statics: Vec<Box<str>>,
        stmts: Vec<StmtTempl>,
    ) -> Self {
        Self {
            layout,
            blocks,
            statics,
            stmts,
        }
    }

    pub fn from_source(source: &Source) -> Result<File> {
        match Parser::new(source.resolve_source()?.as_ref(), SynVisitor::new()).parse() {
            Ok(ok) => Ok(ok.finish()),
            Err(ParseError::Generic(err)) => error!("{err}"),
        }
    }
}

/// Contains a single file template information.
///
/// This template can represent either main template or layout.
pub struct Template {
    meta: Metadata,
    file: File,
}

impl Template {
    pub fn new(meta: Metadata, file: File) -> Self {
        Self { meta, file }
    }

    /// Returns selected block if any, otherwise return all statements..
    pub fn stmts(&self) -> Result<&[StmtTempl]> {
        match self.meta.block.as_ref() {
            Some(block) => Ok(&self.get_block(block)?.stmts),
            None => Ok(&self.file.stmts),
        }
    }

    pub fn get_block(&self, block: &Ident) -> Result<&BlockContent> {
        self.file
            .blocks
            .iter()
            .find(|e| &e.templ.name == block)
            .ok_or_else(|| Error::new(block.span(), format!("cannot find block `{block}`")))
    }

    pub fn path(&self) -> Option<&str> {
        self.meta.path.as_deref()
    }

    pub fn into_layout(self) -> Option<LayoutTempl> {
        self.file.layout
    }

    /// Returns all static contents in template.
    pub fn statics(&self) -> &[Box<str>] {
        &self.file.statics
    }

    pub fn reload(&self) -> &Reload {
        &self.meta.reload
    }

    pub fn blocks(&self) -> &[BlockContent] {
        &self.file.blocks
    }
}

pub struct BlockContent {
    pub templ: BlockTempl,
    pub stmts: Vec<StmtTempl>,
}

