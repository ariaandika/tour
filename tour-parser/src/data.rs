use syn::*;

use crate::{
    ast::StmtTempl,
    file::{AliasKind, BlockContent, File, Import},
    metadata::Metadata,
    syntax::LayoutTempl,
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
    /// Create new [`Template`].
    pub fn new(meta: Metadata, file: File) -> Result<Self> {
        let me = Self { meta, file };
        me.try_stmts()?;
        Ok(me)
    }

    /// Returns selected block if any, otherwise return all statements.
    pub fn try_stmts(&self) -> Result<&[StmtTempl]> {
        match self.meta.block() {
            Some(block) => Ok(&self.try_block(block)?.stmts),
            None => Ok(self.file.stmts()),
        }
    }

    pub(crate) fn stmts(&self) -> &[StmtTempl] {
        self.try_stmts().expect("[BUG] validation missed, selected block missing")
    }

    fn get_import_by_alias(&self, key: &Ident) -> Option<&Import> {
        self.file
            .imports()
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
            .imports()
            .iter()
            .find(|&e|e == &*path)
            .ok_or_else(|| Error::new(key.span(), format!("cannot find template `{}`",path)))
    }

    fn get_block(&self, block: &Ident) -> Option<&BlockContent> {
        self.file
            .blocks()
            .iter()
            .find(|e| &e.templ.name == block)
    }

    pub fn try_block(&self, block: &Ident) -> Result<&BlockContent> {
        self.get_block(block)
            .ok_or_else(|| Error::new(block.span(), format!("cannot find block `{block}`")))
    }

    pub fn meta(&self) -> &Metadata {
        &self.meta
    }

    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn into_parts(self) -> (Metadata, File) {
        (self.meta,self.file)
    }

    pub fn into_layout(self) -> Option<LayoutTempl> {
        self.file.into_layout()
    }
}

