//! The [`Template`] struct.
use syn::*;
use crate::{ast::StmtTempl, file::File, metadata::Metadata};
mod validate;

/// Contains a single file template information.
#[derive(Debug)]
pub struct Template {
    name: Ident,
    meta: Metadata,
    file: File,
}

impl Template {
    /// Create new [`Template`].
    pub fn new(name: Ident, meta: Metadata, file: File) -> Result<Self> {
        let mut me = Self { name, meta, file };
        validate::validate(&mut me)?;
        Ok(me)
    }

    /// Returns selected block if any, otherwise return all statements.
    pub(crate) fn stmts(&self) -> &[StmtTempl] {
        match self.meta.block() {
            Some(block) => match self.file.get_block(block) {
                Some(block) => &block.stmts,
                None => panic!("[BUG] validation missed, selected block missing"),
            },
            None => self.file.stmts(),
        }
    }

    /// Returns all statements, regardles selected block.
    pub fn all_stmts(&self) -> &[StmtTempl] {
        self.file.stmts()
    }

    /// Returns template name.
    ///
    /// Template name is from either derive macro ident, aliased, or auto generated.
    pub fn name(&self) -> &Ident {
        &self.name
    }

    /// Returns template [`Metadata`].
    pub fn meta(&self) -> &Metadata {
        &self.meta
    }

    /// Returns template [`File`].
    pub fn file(&self) -> &File {
        &self.file
    }

    /// Split template into parts.
    pub fn into_parts(self) -> (Metadata, File) {
        (self.meta,self.file)
    }
}

