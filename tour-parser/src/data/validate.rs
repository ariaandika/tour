use std::rc::Rc;
use quote::format_ident;
use syn::*;
use tour_core::Delimiter;

use super::Template;
use crate::{ast::*, common::error, file::BlockContent, syntax::*};

pub fn validate(templ: &mut Template) -> Result<()> {
    templ.try_stmts()?;

    if let Some(block) = templ.file.get_block(&quote::format_ident!("TourInner")) {
        error!(block.templ.name, "`TourInner` is reserved block name")
    }

    if let Some(layout) = templ.file.layout() {
        let import = templ.file.import_by_path(&layout.path);
        let name = import.templ().name().clone();

        let mut inner = vec![
            StmtTempl::Scalar(Scalar::Expr {
                expr: Rc::new(syn::parse_quote!(#name(self))),
                delim: Delimiter::Bang,
            })
        ];

        std::mem::swap(templ.file.stmts_mut(), &mut inner);

        templ.file.blocks_mut().push(BlockContent {
            templ: BlockTempl {
                pub_token: Some(<_>::default()),
                static_token: Some(<_>::default()),
                block_token: <_>::default(),
                name: format_ident!("TourInner"),
            },
            stmts: inner,
        });
    }


    Ok(())
}

