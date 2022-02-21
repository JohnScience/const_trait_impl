use crate::WhereClause;
use syn::{parse::ParseStream, Result};

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
