use proc_macro::TokenStream;
use quote::quote;

use crate::parser::{Table, fields_named_struct, get_sql_columns};

pub fn sql_select_all_macro_derive(input: &mut syn::DeriveInput) -> TokenStream {
    let table = Table::parse(input);
    let struct_name = input.ident.clone();
    let table_name = table.get_name();

    let fields = fields_named_struct(input);
    let sql_columns = get_sql_columns(fields).join(", ");

    let query = format!("SELECT {sql_columns} FROM {table_name}");

    let token_stream = quote! {
        impl #struct_name {
            #[doc=#query]
            pub async fn select_all(pool: &sqlx::PgPool) -> Result<Vec<#struct_name>, sqlx::Error> {
                let object = sqlx::query_as!(#struct_name, #query)
                    .fetch_all(pool)
                    .await?;
                Ok(object)
            }
        }
    };
    token_stream.into()
}
