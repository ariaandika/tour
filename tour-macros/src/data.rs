use syn::*;

use crate::{
    shared::Reload,
    syntax::{BlockTempl, LayoutTempl},
    visitor::StmtTempl,
};

/// Extra information declared outside template file.
pub struct Metadata {
    path: Option<Box<str>>,
    reload: Reload,
    block: Option<Ident>,
}

impl Metadata {
    /// Create new [`Metadata`].
    pub fn new(path: Option<Box<str>>, reload: Reload, block: Option<Ident>) -> Self {
        Self {
            path,
            reload,
            block,
        }
    }

    pub fn from_layout(layout: LayoutTempl, reload: Reload) -> Result<Metadata> {
        Ok(Self {
            path: crate::shared::SourceTempl::from_layout(&layout)?.resolve_path(),
            reload,
            block: None, // TODO: allows select block for a layout
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
}

pub struct BlockContent {
    #[allow(unused)]
    pub templ: BlockTempl,
    pub stmts: Vec<StmtTempl>,
}

