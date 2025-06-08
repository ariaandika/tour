use std::{borrow::Cow, fs::read_to_string};
use quote::format_ident;
use syn::*;

use crate::{
    common::{Reload, error, path},
    config::Config,
    file::File,
    syntax::{BlockTempl, LayoutTempl},
    visitor::StmtTempl,
};

// ===== Template =====

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

    /// Returns selected block if any, otherwise return all statements.
    pub fn stmts(&self) -> Result<&[StmtTempl]> {
        match self.meta.block.as_ref() {
            Some(block) => Ok(&self.try_block(block)?.stmts),
            None => Ok(self.file.stmts()),
        }
    }

    /// This will look for a block, imported template, and block inside the imported template.
    pub fn resolve_alias<'me>(&'me self, key: &Path) -> Result<AliasKind<'me>> {
        let mut iter = key.segments.iter();
        let d1 = &error!(?iter.next(),"path empty").ident;

        if let Some(block) = self.get_block(d1) {
            return Ok(AliasKind::Block(block))
        }

        let import = self.try_import_by_alias(d1)?;

        let Some(d2) = iter.next() else {
            return self.try_import_by_alias(d1).map(AliasKind::Import);
        };

        let d2 = &d2.ident;
        error!(.iter.next().is_some(),"only 2 level path is supported");

        match import.templ().get_block(d2) {
            Some(_block) => todo!("how to render, block from other template ?"),// Ok(AliasKind::Import(())),
            None => error!("cannot find block `{d2}` in `{d1}`"),
        }
    }

    fn get_import_by_alias(&self, key: &Ident) -> Option<&Import> {
        self.file
            .imports
            .iter()
            .find(|&e|e == key)
    }

    fn try_import_by_alias(&self, key: &Ident) -> Result<&Import> {
        self.get_import_by_alias(key).ok_or_else(|| {
            Error::new(
                key.span(),
                format!("cannot find block/import `{key}`"),
            )
        })
    }

    pub fn try_import_by_path(&self, key: &LitStr) -> Result<&Import> {
        let path = key.value();
        self.file
            .imports
            .iter()
            .find(|&e|e == &*path)
            .ok_or_else(|| Error::new(key.span(), format!("cannot find template `{}`",path)))
    }

    fn get_block(&self, block: &Ident) -> Option<&BlockContent> {
        self.file
            .blocks
            .iter()
            .find(|e| &e.templ.name == block)
    }

    pub fn try_block(&self, block: &Ident) -> Result<&BlockContent> {
        self.get_block(block)
            .ok_or_else(|| Error::new(block.span(), format!("cannot find block `{block}`")))
    }

    pub fn path(&self) -> &str {
        self.meta.path.as_ref()
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

    pub fn imports(&self) -> &[Import] {
        &self.file.imports
    }

    /// Returns `true` if template is a file, not inlined.
    pub fn is_file(&self) -> bool {
        self.meta.is_file()
    }

    pub fn into_parts(self) -> (Metadata, File) {
        (self.meta,self.file)
    }

    pub fn into_layout(self) -> Option<LayoutTempl> {
        self.file.layout
    }
}

