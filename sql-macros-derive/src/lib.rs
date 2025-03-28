use proc_macro::TokenStream;

mod delete;
mod insert;
mod insert_many;
mod parser;
mod select;
mod select_all;
mod select_many;
mod table;
mod update;

#[proc_macro_derive(SqlSelect, attributes(table))]
pub fn sql_select_macro_derive(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    select::sql_select_macro_derive(&mut input)
}

#[proc_macro_derive(SqlSelectAll, attributes(table))]
pub fn sql_select_all_macro_derive(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    select_all::sql_select_all_macro_derive(&mut input)
}

#[proc_macro_derive(SqlSelectMany, attributes(table))]
pub fn sql_select_many_macro_derive(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    select_many::sql_select_many_macro_derive(&mut input)
}

#[proc_macro_derive(SqlInsert, attributes(table))]
pub fn sql_insert_macro_derive(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    insert::sql_insert_macro_derive(&mut input)
}

#[proc_macro_derive(SqlInsertMany, attributes(table))]
pub fn sql_insert_many_macro_derive(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    insert_many::sql_insert_many_macro_derive(&mut input)
}

#[proc_macro_derive(SqlUpdate, attributes(table))]
pub fn sql_update_macro_derive(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    update::sql_update_macro_derive(&mut input)
}

#[proc_macro_derive(SqlDelete, attributes(table))]
pub fn sql_delete_macro_derive(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    delete::sql_delete_macro_derive(&mut input)
}

#[proc_macro_derive(SqlTable, attributes(table))]
pub fn sql_table_macro_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    table::sql_table_macro_derive(&input).unwrap()
}
