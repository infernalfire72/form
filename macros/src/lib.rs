use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident};

mod tools;
use tools::Constraint;

fn generate_select_builder_impl(model_ident: &Ident, fields: &Vec<tools::Field>) -> TokenStream2 {
    let ident = format_ident!("__FormQb{}", model_ident);

    quote! {
        pub struct #ident {
            #(pub #fields: form::Operational,)*
        }

        impl #ident {
            pub fn default() -> Self {
                Self {
                    #(#fields: form::Operational(stringify!(#fields)),)*
                }
            }
        }

        impl #model_ident {
            pub fn select() -> form::SelectQuery<Self, #ident> {
                form::SelectQuery::new(Self::BASE_SELECT)
            }

            pub fn wselect<F>(lambda: F) -> form::SelectQuery<Self, #ident>
            where F: Fn(#ident) -> form::Operation {
                let op = lambda(#ident::default());
                let (clause, params) = op.format();

                form::SelectQuery::with_params(
                    Self::BASE_SELECT,
                    params,
                    vec![form::WhereClause::Single(clause)],
                )
            }
        }
    }
}

fn generate_unique_fetchers(
    self_ident: &Ident,
    table_name: &String,
    fields: &Vec<tools::Field>,
    primary_keys: &Vec<&tools::Field>,
    unique_keys: &Vec<&tools::Field>,
) -> TokenStream2 {
    let base_select =
        quote! { concat!("SELECT ", stringify!(#(#fields),*), " FROM ", #table_name) };
    let unique_impls = unique_keys.iter().chain(primary_keys).map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        let find_name = format_ident!("find_{}", ident);
        quote! {
            pub async fn #find_name (#ident: #ty, pool: &form::Pool) -> form::Result<#self_ident> {
                form::query_as::<_, #self_ident>(concat!(#base_select, " WHERE ", stringify!(#ident), " = ?"))
                    .bind(#ident)
                    .fetch_one(pool)
                    .await
            }
        }
    });

    if primary_keys.len() > 1 {
        let primary_tys = primary_keys.iter().map(|field| field.ty.clone());
        let primaries_joined = primary_keys
            .iter()
            .map(|field| field.ident.to_string())
            .collect::<Vec<_>>()
            .join("_");
        let find_primary = format_ident!("find_{}", primaries_joined);

        quote! {
            #(#unique_impls)*
            pub async fn #find_primary (#(#primary_keys: #primary_tys,)* pool: &form::Pool) -> form::Result<#self_ident> {
                form::query_as::<_, #self_ident>(concat!(#base_select, " WHERE ", stringify!(#(#primary_keys = ?) AND *)))
                    #(.bind(#primary_keys))*
                    .fetch_one(pool)
                    .await
            }
        }
    } else {
        quote! { #(#unique_impls)* }
    }
}

fn queryable_impl(ident: &Ident, struct_data: &DataStruct) -> TokenStream2 {
    let table_name = format!("{}s", ident.to_string().to_lowercase());

    let fields = match struct_data.fields {
        Fields::Named(ref fields) => &fields.named,
        _ => unimplemented!(),
    };
    let parsed_fields = tools::parse_fields(fields);

    let select_builder_impl = generate_select_builder_impl(ident, &parsed_fields);

    let primary_keys = tools::get_primary_keys(&parsed_fields);
    let unique_keys = tools::get_unique_keys(&parsed_fields);
    let unique_fetchers = generate_unique_fetchers(
        ident,
        &table_name,
        &parsed_fields,
        &primary_keys,
        &unique_keys,
    );

    let question_marks = parsed_fields
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let always_insert_fields = parsed_fields
        .iter()
        .filter(|field| !field.is_serial && field.default.is_none())
        .collect::<Vec<_>>();
    let optional_insert_fields = parsed_fields
        .iter()
        .filter(|field| field.is_serial || field.default.is_some())
        .collect::<Vec<_>>();
    let optional_insert_field_tys = optional_insert_fields.iter().map(|field| field.ty.clone());
    let insert_field_order = always_insert_fields
        .iter()
        .chain(optional_insert_fields.iter())
        .collect::<Vec<_>>();
    let update_fields = parsed_fields
        .iter()
        .filter(|field| field.constraint != Constraint::Primary)
        .collect::<Vec<_>>();

    quote! {
        #select_builder_impl

        impl form::FromRow<'_, form::Row> for #ident {
            fn from_row(row: &form::Row) -> form::Result<Self> {
                use form::RowLike as _;
                Ok(Self {
                    #(#parsed_fields: row.try_get(stringify!(#parsed_fields))?,)*
                    ..Default::default()
                })
            }
        }

        impl #ident {
            const TABLE_NAME: &'static str = #table_name;
            pub fn primary_keys() -> &'static [&'static str] { &[#(stringify!(#primary_keys), )*] }
            const PRIMARY_WHERE: &'static str = stringify!(#(#primary_keys = ?) AND *);
            pub fn read_params() -> &'static [&'static str] { &[#(stringify!(#parsed_fields), )*] }
            const BASE_SELECT: &'static str = concat!("SELECT ", stringify!(#(#parsed_fields),*), " FROM ", #table_name);

            pub async fn create<'a>(&self, executor: impl form::Executor<'a, Database = form::Protocol>) -> form::Result<form::QueryResult> {
                #(
                    let #optional_insert_fields =
                        if self.#optional_insert_fields != #optional_insert_field_tys::default() {
                            Some(&self.#optional_insert_fields)
                        } else {
                            None
                        };
                )*

                let raw_query = concat!("INSERT INTO ", #table_name, " (", stringify!(#(#insert_field_order),*), ") VALUES (", #question_marks, ")");
                form::query(raw_query)
                    #(.bind(&self.#always_insert_fields))*
                    #(.bind(#optional_insert_fields))*
                    .execute(executor)
                    .await
            }

            pub async fn update<'a>(&self, executor: impl form::Executor<'a, Database = form::Protocol>) -> form::Result<form::QueryResult> {
                let raw_query = concat!("UPDATE ", #table_name, " SET ", stringify!(#(#update_fields = ?),* WHERE #(#primary_keys = ?) AND *));
                form::query(raw_query)
                    #(.bind(&self.#update_fields))*
                    #(.bind(&self.#primary_keys))*
                    .execute(executor)
                    .await
            }

            pub async fn delete<'a>(&self, executor: impl form::Executor<'a, Database = form::Protocol>) -> form::Result<form::QueryResult> {
                let raw_query = concat!("DELETE FROM ", #table_name, " WHERE ", stringify!(#(#primary_keys = ?) AND *));
                form::query(raw_query)
                    #(.bind(&self.#primary_keys))*
                    .execute(executor)
                    .await
            }

            #unique_fetchers
        }
    }
}

#[proc_macro_derive(
    Queryable,
    attributes(serial, primary_key, unique, suppress, default_value, sql_type)
)]
pub fn queryable(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let s_ident = &input.ident;

    match input.data {
        Data::Struct(ref struct_data) => TokenStream::from(queryable_impl(s_ident, struct_data)),
        _ => unimplemented!(),
    }
}
