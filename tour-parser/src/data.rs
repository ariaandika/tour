use std::{borrow::Cow, fs::read_to_string};
use quote::format_ident;
use syn::*;

use crate::{
    common::{Reload, error, path},
    config::Config,
    syntax::{BlockTempl, LayoutTempl},
    visitor::StmtTempl,
};

// ===== File =====

/// Content of a template file.
pub struct File {
    layout: Option<LayoutTempl>,
    imports: Vec<Import>,
    blocks: Vec<BlockContent>,
    statics: Vec<Box<str>>,
    stmts: Vec<StmtTempl>,
}

pub struct BlockContent {
    pub templ: BlockTempl,
    pub stmts: Vec<StmtTempl>,
}

impl File {
    pub fn new(
        layout: Option<LayoutTempl>,
        imports: Vec<Import>,
        blocks: Vec<BlockContent>,
        statics: Vec<Box<str>>,
        stmts: Vec<StmtTempl>,
    ) -> Self {
        Self {
            layout,
            imports,
            blocks,
            statics,
            stmts,
        }
    }

    pub fn from_meta(meta: &Metadata) -> Result<File> {
        crate::visitor::generate_file(meta)
    }

    pub fn stmts(&self) -> &[StmtTempl] {
        &self.stmts
    }

    pub fn into_layout(self) -> Option<LayoutTempl> {
        self.layout
    }
}


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

// ===== ImportKey =====

pub struct Import {
    path: Box<str>,
    alias: Option<Ident>,
    templ: Template,
}

impl Import {
    pub fn new(path: Box<str>, alias: Option<Ident>, templ: Template) -> Self {
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

