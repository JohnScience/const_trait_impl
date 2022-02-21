use crate::WhereClause;
use syn::{parse::ParseStream, Result, Visibility};

pub(super) trait LocalParse: Sized {
    fn local_parse(input: ParseStream) -> Result<Self>;
}

impl LocalParse for Option<WhereClause> {
    fn local_parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::token::Where) {
            input.parse().map(Some)
        } else {
            Ok(None)
        }
    }
}

pub(super) trait LocalIsInherited {
    fn local_is_inherited(&self) -> bool;
}

impl LocalIsInherited for Visibility {
    fn local_is_inherited(&self) -> bool {
        match *self {
            Visibility::Inherited => true,
            _ => false,
        }
    }
}
