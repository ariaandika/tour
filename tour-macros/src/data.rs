use std::{borrow::Cow, fs::read_to_string};
use syn::{spanned::Spanned, *};

use crate::{
    common::{Reload, error, path},
    config::Config,
    syntax::{BlockTempl, LayoutTempl},
    visitor::{Import, StmtTempl},
};

/// Extra information declared outside template file.
#[derive(Debug)]
pub struct Metadata {
    path: Box<str>,
    source: Option<Box<str>>,
    reload: Reload,
    block: Option<Ident>,
}

impl Metadata {
    pub fn new(path: Box<str>, source: Option<Box<str>>, reload: Reload, block: Option<Ident>) -> Self {
        Self {
            path,
            source,
            reload,
            block,
        }
    }

    pub fn from_attrs(attrs: &[Attribute], conf: &Config) -> Result<Metadata> {
        crate::attribute::generate_meta(attrs, conf)
    }

    /// Generate inherited [`Metadata`] with given path.
    pub fn clone_with_path(&self, path: impl AsRef<std::path::Path>) -> Metadata {
        Self {
            path: path::resolve_at(path, self.dir_ref()),
            source: None,
            reload: self.reload.clone(),
            block: None,
        }
    }

    /// Generate layout [`Metadata`] inherited from parent meta.
    pub fn clone_with_layout(&self, layout: &LayoutTempl) -> Metadata {
        Self {
            path: path::resolve_at(layout.path.value(), self.dir_ref()),
            source: None,                // there is no inline layout
            reload: self.reload.clone(), // layout specific reload seems redundant
            block: None,                 // allows select block for a layout ?
        }
    }

    pub fn resolve_source(&self) -> Result<Cow<'_, str>> {
        match self.source.as_deref() {
            Some(src) => Ok(src.into()),
            None => Ok(error!(
                !read_to_string(&*self.path),
                "cannot read `{}`: {}", self.path
            )
            .into()),
        }
    }

    pub fn dir_ref(&self) -> &std::path::Path {
        std::path::Path::new(&*self.path)
            .parent()
            .unwrap_or(std::path::Path::new("/"))
    }

    /// Returns `true` if template is a file, not inlined.
    pub fn is_file(&self) -> bool {
        std::path::Path::new(&*self.path).is_file()
    }
}

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
            Some(block) => Ok(&self.get_block(block)?.stmts),
            None => Ok(self.file.stmts()),
        }
    }

    #[allow(unused, reason = "used later")]
    pub fn get_import_by_alias(&self, key: &Path) -> Result<&Template> {
        self.file
            .imports
            .iter()
            .find(|&e|e == key)
            .map(|e|&e.templ)
            .ok_or_else(|| Error::new(key.span(), format!("cannot find template `{}`",fmt_path(key))))
    }

    pub fn get_import_by_path(&self, key: &LitStr) -> Result<&Template> {
        let path = key.value();
        self.file
            .imports
            .iter()
            .find(|&e|e == &*path)
            .map(|e|&e.templ)
            .ok_or_else(|| Error::new(key.span(), format!("cannot find template `{}`",path)))
    }

    pub fn get_block(&self, block: &Ident) -> Result<&BlockContent> {
        self.file
            .blocks
            .iter()
            .find(|e| &e.templ.name == block)
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

fn fmt_path(path: &Path) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for seg in &path.segments {
        let _ = write!(s, "{}", seg.ident);
    }
    s
}

