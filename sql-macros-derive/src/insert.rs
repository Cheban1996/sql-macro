use proc_macro::TokenStream;
use quote::quote;

use crate::parser::{Table, fields_named_struct, get_sql_columns};

pub fn sql_insert_macro_derive(input: &mut syn::DeriveInput) -> TokenStream {
    let table = Table::parse(input);
    let struct_name = input.ident.clone();
    let table_name = table.get_name();

    let fields = fields_named_struct(input);

    let sql_columns = get_sql_columns(fields).join(", ");
    let idents = fields
        .iter()
        .filter(|field| field.ident.is_some())
        .map(|field| field.clone().ident.unwrap());
    let sql_column_index = idents
        .clone()
        .enumerate()
        .map(|(index, _)| format!("${}", index + 1))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!("INSERT INTO {table_name} ({sql_columns}) VALUES ({sql_column_index})");
    let returning = table.get_return_fields().unwrap_or("*".to_string());

    let token_stream = if let Some(return_type) = table.get_return_type() {
        let type_param =
            syn::parse_str::<syn::Type>(&return_type).expect("Failed to parse input string");

        let query = format!("{query} RETURNING {returning}");

        quote! {
            impl #struct_name {
                #[doc=#query]
                pub async fn insert(&self, conn: &mut sqlx::PgConnection) -> Result<#type_param, sqlx::Error>
                {
                    let object = sqlx::query_as!(
                        #type_param,
                        #query,
                        #(self.#idents as _),* // Example: fields as _
                    )
                    .fetch_one(&mut *conn)
                    .await?;
                    Ok(object)
                }
            }
        }
    } else {
        quote! {
            impl #struct_name {
                #[doc=#query]
                pub async fn insert(&self, conn: &mut sqlx::PgConnection) -> Result<sqlx::any::AnyQueryResult, sqlx::Error>
                {
                    let query_result = sqlx::query!(
                        #query,
                        #(self.#idents as _),* // Example: fields as _
                    )
                    .execute(&mut *conn)
                    .await?;
                    Ok(query_result.into())
                }
            }
        }
    };
    token_stream.into()
}
