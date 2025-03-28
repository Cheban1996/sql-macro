use proc_macro2::{Ident, TokenTree};
use quote::ToTokens;
use syn::{Meta, MetaList, PathArguments, Type};

fn syntax_tree_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let mut data = vec![];
            for segment in &path.segments {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() == 1
                        && let syn::GenericArgument::Type(inner_ty) = &args.args[0]
                    {
                        let inner_type_name = syntax_tree_to_string(inner_ty);
                        data.push(format!("{}<{}>", segment.ident, inner_type_name));
                    }
                } else {
                    data.push(segment.ident.to_string());
                }
            }
            data.join("::")
        }
        _ => panic!("Unexpected type"),
    }
}

pub fn fields_named_struct(
    input: &syn::DeriveInput,
) -> &syn::punctuated::Punctuated<syn::Field, syn::token::Comma> {
    match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => {
            panic!("Only for named struct");
        }
    }
}

pub fn parse_fields_with_type(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    token_stream_ident: &str, // select, delete
) -> Vec<(Ident, syn::Type)> {
    let mut stamp: Vec<(Ident, syn::Type)> = Vec::new(); // field_name, field_type
    for field in fields {
        for attr in &field.attrs {
            match attr.clone().meta {
                syn::Meta::List(meta) => {
                    if !meta
                        .path
                        .segments
                        .iter()
                        .any(|path_segment| path_segment.ident == "table")
                    {
                        continue;
                    }
                    if !meta.tokens.into_iter().any(|token| match token {
                        TokenTree::Ident(ident) => ident == token_stream_ident,
                        _ => false,
                    }) {
                        continue;
                    }
                }
                _ => continue,
            }

            let field_name = field.ident.clone().expect("expect Ident but get None");
            let data = syntax_tree_to_string(&field.clone().ty);
            let type_param =
                syn::parse_str::<syn::Type>(&data).expect("Failed to parse input string");

            stamp.push((field_name, type_param));
        }
    }
    stamp
}

pub fn get_method_params(fields_with_type: Vec<(Ident, syn::Type)>) -> proc_macro2::TokenStream {
    let method_params = fields_with_type
        .clone()
        .iter()
        .map(|(name_param, type_param)| format!("{name_param}: {}", type_param.to_token_stream()))
        .collect::<Vec<String>>()
        .join(", ");
    syn::parse_str::<proc_macro2::TokenStream>(&method_params).expect("Failed to parse code string")
}

pub fn get_filters(fields_with_type: Vec<(Ident, syn::Type)>) -> Vec<proc_macro2::Ident> {
    fields_with_type
        .clone()
        .iter()
        .map(|(name_param, _)| name_param.clone())
        .collect::<Vec<Ident>>()
}

pub fn generate_sql_params_condition(name_params: &[proc_macro2::Ident]) -> String {
    name_params
        .iter()
        .enumerate()
        .map(|(item, name_param)| format!("{name_param}=${}", item + 1))
        .collect::<Vec<String>>()
        .join(" AND ")
}

pub fn get_sql_columns(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<String> {
    fields
        .iter()
        .filter_map(|field| {
            let field_name = field.clone().ident.as_ref()?.to_string();

            for attr in &field.attrs {
                if !attr.path().is_ident("table") {
                    continue;
                }

                if let syn::Meta::List(meta_list) = &attr.meta {
                    let mut current_key = false;

                    for token in meta_list.clone().tokens.into_iter() {
                        match token {
                            TokenTree::Ident(ident) => {
                                if ident == "as_type" {
                                    current_key = true;
                                }
                            }
                            TokenTree::Literal(literal) => {
                                if !current_key {
                                    continue;
                                }
                                return Some(format!("{field_name} AS {literal}")); // Example: "email, role AS \"role!: Role\""
                            }
                            _ => {}
                        }
                    }
                }
            }
            Some(field_name)
        })
        .collect()
}

pub fn get_struct_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<String> {
    fields
        .iter()
        .filter_map(|field| Some(field.clone().ident.as_ref()?.to_string()))
        .collect()
}

pub fn extract_fields_as_params(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<(Ident, syn::Type)> {
    let mut stamp: Vec<(Ident, syn::Type)> = Vec::new(); // field_name, field_type
    for field in fields {
        let field_name = field
            .ident
            .clone()
            .expect("Struct must by named type struct");
        let data = syntax_tree_to_string(&field.clone().ty);
        let type_param = syn::parse_str::<syn::Type>(&data).expect("Failed to parse input string");
        stamp.push((field_name, type_param));
    }
    if stamp.is_empty() {
        panic!("Struct is empty")
    }
    stamp
}

/// Parse `#[table(some_ident = some_value)]` by some_ident and return Some(some_value)
fn get_kind_str(meta: &MetaList, by_ident: &str) -> Option<String> {
    let mut kind: Option<String> = None;
    let mut read_next_literal = false;

    for token in meta.tokens.clone() {
        match token {
            TokenTree::Ident(ident) => {
                if read_next_literal {
                    kind = Some(ident.to_string());
                    break;
                }

                if ident != by_ident {
                    continue;
                }
                read_next_literal = true;
            }
            TokenTree::Literal(literal) => {
                if read_next_literal {
                    kind = Some(literal.to_string());
                    break;
                }
            }
            _ => continue,
        }
    }
    kind.map(|kind| kind.replace('"', ""))
}

/// Parse `#[table(some_ident = some_method(field1, field1))]` by some_ident and return Some((some_method, vec![field1, field1]))
///
/// `#[table(some_ident1 = some_method(field1, field1), some_ident2 = some_method(field1, field1))]`
/// some_ident2 will not be parsed
fn get_kind_method_with_params(meta: &MetaList, by_ident: &str) -> Option<(String, Vec<String>)> {
    let mut kind: Option<(String, Vec<String>)> = None;
    let mut read_next_literal = false;
    let mut read_next_group = false;

    for token in meta.tokens.clone() {
        match token {
            TokenTree::Ident(ident) => {
                if ident == by_ident {
                    read_next_literal = true;
                    read_next_group = true;
                    continue;
                }
                if read_next_literal {
                    kind = Some((ident.to_string(), vec![]));
                    read_next_literal = false;
                }
            }
            TokenTree::Group(group) => {
                if read_next_group {
                    if let Some(k) = &mut kind {
                        let params = group
                            .stream()
                            .into_iter()
                            .filter(|token_stream| matches!(token_stream, TokenTree::Ident(_)))
                            .map(|token_stream| token_stream.to_string())
                            .collect::<Vec<String>>();
                        k.1 = params;
                    }
                    read_next_group = false;
                }
            }
            _ => continue,
        }
    }
    kind
}

pub struct Table {
    struct_name: Ident,
    meta_list: Vec<MetaList>,
}

impl Table {
    pub fn parse(input: &syn::DeriveInput) -> Table {
        let meta_list = input
            .attrs
            .iter()
            .filter_map(|attr| match attr.meta.clone() {
                Meta::List(meta) if meta.path.is_ident("table") => Some(meta),
                _ => None,
            })
            .collect::<Vec<_>>();

        Table {
            struct_name: input.ident.clone(),
            meta_list,
        }
    }

    /// Use for extract table name `#[table(name = users)]` else return struct name
    pub fn get_name(&self) -> String {
        self.meta_list
            .iter()
            .find_map(|meta| get_kind_str(meta, "name"))
            .unwrap_or(format!("{}s", self.struct_name).to_lowercase())
    }

    /// Use for extract return special columns for update `#[table(spec_columns = "updated_at=NOW()")]`
    pub fn get_spec_columns(&self) -> Option<String> {
        self.meta_list
            .iter()
            .find_map(|meta| get_kind_str(meta, "spec_columns"))
    }

    /// Use for extract return type `#[table(return_type = User)]`
    pub fn get_return_type(&self) -> Option<String> {
        self.meta_list
            .iter()
            .find_map(|meta| get_kind_str(meta, "return_type"))
    }

    /// Use for extract return type `#[table(return_fields = "id, user_id")]`
    pub fn get_return_fields(&self) -> Option<String> {
        self.meta_list
            .iter()
            .find_map(|meta| get_kind_str(meta, "return_fields"))
    }


    /// Use for extract methods select `#[table(select = get_active_user(is_active, is_removed))]`
    pub fn get_select(&self) -> Vec<(String, Vec<String>)> {
        self.meta_list
            .iter()
            .filter_map(|meta| get_kind_method_with_params(meta, "select"))
            .collect()
    }

    /// Use for extract methods select_many `#[table(select_many = get_user_by_removed(is_active, is_removed))]`
    pub fn get_select_many(&self) -> Vec<(String, Vec<String>)> {
        self.meta_list
            .iter()
            .filter_map(|meta| get_kind_method_with_params(meta, "select_many"))
            .collect()
    }

    /// Use for extract methods delete `#[table(delete = delete_by_user(id, user_id))]`
    pub fn get_delete(&self) -> Vec<(String, Vec<String>)> {
        self.meta_list
            .iter()
            .filter_map(|meta| get_kind_method_with_params(meta, "delete"))
            .collect()
    }

    // /// Use for extract methods delete `#[table(update = update_by_user(id, user_id))]`
    // pub fn get_update(&self) -> Vec<(String, Vec<String>)> {
    //     self.meta_list
    //         .iter()
    //         .filter_map(|meta| get_kind_method_with_params(meta, "delete"))
    //         .collect()
    // }
}
