use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, ExprArray, ExprLit, Lit, Meta, Token,
};

#[must_use]
pub fn sanitize_name(s: String) -> String {
    match s.as_str() {
        "const" | "enum" | "ref" | "type" => format!("r#{s}"),
        _ => s,
    }
}
/// Configuration for the `suite` attribute.
pub struct SuiteConfig {
    pub path: String,
    pub drafts: Vec<String>,
    pub xfail: Vec<String>,
}

impl Parse for SuiteConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path = None;
        let mut drafts = Vec::new();
        let mut xfail = Vec::new();

        for meta in Punctuated::<Meta, Token![,]>::parse_terminated(input)? {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("path") => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit), ..
                    }) = nv.value
                    {
                        path = Some(lit.value());
                    } else {
                        return Err(syn::Error::new_spanned(
                            nv.value,
                            "Test suite path should be a string literal",
                        ));
                    }
                }
                Meta::NameValue(nv) if nv.path.is_ident("drafts") => {
                    if let Expr::Array(ExprArray { elems, .. }) = nv.value {
                        for elem in elems {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }) = elem
                            {
                                drafts.push(lit.value());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    elem,
                                    "Drafts name should be a string literal",
                                ));
                            }
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            nv.value,
                            "Drafts should be an array of string literals",
                        ));
                    }
                }
                Meta::NameValue(nv) if nv.path.is_ident("xfail") => {
                    if let Expr::Array(ExprArray { elems, .. }) = nv.value {
                        for elem in elems {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }) = elem
                            {
                                xfail.push(lit.value());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    elem,
                                    "XFail item should be a string literal",
                                ));
                            }
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            nv.value,
                            "XFail should be an array of string literals",
                        ));
                    }
                }
                _ => return Err(syn::Error::new_spanned(meta, "Unexpected attribute")),
            }
        }
        let path = path.ok_or_else(|| {
            syn::Error::new(input.span(), "Missing path to JSON Schema test suite")
        })?;
        if drafts.is_empty() {
            return Err(syn::Error::new(input.span(), "Drafts are missing"));
        }

        Ok(SuiteConfig {
            path,
            drafts,
            xfail,
        })
    }
}
