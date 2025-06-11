use quote::format_ident;
use syn::*;

use super::Template;
use crate::{ast::*, common::{error, INNER_BLOCK}, file::BlockContent, syntax::*};

pub fn validate(templ: &mut Template) -> Result<()> {
    // check if selected block exists
    if let Some(block) = templ.meta.block() {
        if templ.file.get_block(block).is_none() {
            error!("cannot find `{block}` in `{}`",templ.name)
        }
    }

    // no inner block reserved name
    if let Some(block) = templ.file.get_block(&quote::format_ident!("{INNER_BLOCK}")) {
        error!(block.templ.name, "`{INNER_BLOCK}` is reserved block name")
    }

    // if uses layout, make inner body as a block
    if let Some(layout) = templ.file.layout() {
        let name = templ.file.import_by_path(&layout.path).alias();

        let mut inner = vec![
            StmtTempl::Scalar(Scalar::Render(RenderTempl {
                render_token: <_>::default(),
                value: RenderValue::Ident(name.clone()),
                block: None,
            })),
        ];

        std::mem::swap(templ.file.stmts_mut(), &mut inner);

        templ.file.blocks_mut().push(BlockContent {
            templ: BlockTempl {
                pub_token: Some(<_>::default()),
                static_token: Some(<_>::default()),
                block_token: <_>::default(),
                name: format_ident!("{INNER_BLOCK}"),
            },
            stmts: inner,
        });
    }

    Ok(())
}

