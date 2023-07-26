use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{token::Comma, Field as SynField, Ident, Type};

#[derive(PartialEq, Eq, Debug)]
pub enum Constraint {
    Primary,
    Unique,
    None,
}

#[derive(Debug)]
pub struct Field {
    pub constraint: Constraint,
    pub is_serial: bool,
    pub ident: Ident,
    pub ty: Type,
    pub default: Option<()>,
}

impl ToTokens for Field {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        Ident::to_tokens(&self.ident, stream)
    }
}

macro_rules! has_attr {
    ($e:expr; $ee:expr) => {
        $e.attrs
            .iter()
            .find(|attr| attr.path.is_ident($ee))
            .is_some()
    };
}

pub fn parse_fields(fields: &Punctuated<SynField, Comma>) -> Vec<Field> {
    fields
        .iter()
        .filter_map(|field| {
            if has_attr!(field; "suppress") {
                return None;
            }

            let constraint = if has_attr!(field; "primary_key") {
                Constraint::Primary
            } else if has_attr!(field; "unique") {
                Constraint::Unique
            } else {
                Constraint::None
            };
            Some(Field {
                constraint,
                is_serial: has_attr!(field; "serial"),
                ident: field.ident.as_ref().unwrap().clone(),
                ty: field.ty.clone(),
                // TODO: parse default
                default: None,
            })
        })
        .collect()
}

pub fn get_primary_keys<'a>(fields: &'a Vec<Field>) -> Vec<&'a Field> {
    fields
        .iter()
        .filter(|field| field.constraint == Constraint::Primary)
        .collect()
}

pub fn get_unique_keys<'a>(fields: &'a Vec<Field>) -> Vec<&'a Field> {
    fields
        .iter()
        .filter(|field| field.constraint == Constraint::Unique)
        .collect()
}
