use proc_macro::TokenStream;
use quote::quote;

use crate::parser::{Table, fields_named_struct, get_sql_columns, get_struct_fields};

pub fn sql_table_macro_derive(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let table = Table::parse(input);
    let struct_name = input.ident.clone();
    let table_name = table.get_name();

    let fields = fields_named_struct(input);

    let sql_columns = get_sql_columns(fields);
    let struct_fields = get_struct_fields(fields);

    let token_stream = quote! {
        impl sql_macros::SqlTable for #struct_name {
            fn name() -> &'static str {
                #table_name
            }
            fn fields() -> Vec<&'static str> {
                vec![
                    #(#struct_fields),*
                ]
            }
            fn sql_columns() -> Vec<&'static str> {
                vec![
                    #(#sql_columns),*
                ]
            }
        }
    };
    Ok(token_stream.into())
}
