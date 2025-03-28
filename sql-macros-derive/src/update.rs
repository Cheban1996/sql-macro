use proc_macro::TokenStream;
use quote::quote;

use crate::parser::{Table, fields_named_struct, get_filters, parse_fields_with_type};

pub fn sql_update_macro_derive(input: &mut syn::DeriveInput) -> TokenStream {
    let table = Table::parse(input);
    let struct_name = input.ident.clone();
    let table_name = table.get_name();

    let fields = fields_named_struct(input);

    let fields_with_type = parse_fields_with_type(fields, "update");
    let filters = get_filters(fields_with_type.clone());
    let count_columns = fields.len() - filters.len();

    let idents = fields
        .iter()
        .filter(|field| field.ident.is_some())
        .map(|field| field.clone().ident.unwrap());
    let columns = idents.clone().filter(|ident| !filters.contains(ident));

    let sql_column = columns
        .clone()
        .enumerate()
        .map(|(index, column)| format!("{column}=${}", index + 1))
        .collect::<Vec<String>>()
        .join(", ");
    let sql_filters = filters
        .iter()
        .enumerate()
        .map(|(index, column)| format!("{column}=${}", count_columns + index + 1))
        .collect::<Vec<String>>()
        .join(" AND ");

    let spec_columns = table
        .get_spec_columns()
        .map(|spec_columns| format!(", {spec_columns}"))
        .unwrap_or_default();

    let query = format!("UPDATE {table_name} SET {sql_column}{spec_columns} WHERE {sql_filters}");
    let returning = table.get_return_fields().unwrap_or("*".to_string());

    let token_stream = if let Some(return_type) = table.get_return_type() {
        let type_param =
            syn::parse_str::<syn::Type>(&return_type).expect("Failed to parse input string");

        let query = format!("{query} RETURNING {returning}");

        quote! {
            impl #struct_name {
                #[doc=#query]
                pub async fn update(&self, conn: &mut sqlx::PgConnection) -> Result<#type_param, sqlx::Error>
                {
                    let object = sqlx::query_as!(
                        #type_param,
                        #query,
                        #(self.#columns as _),*, // Example: fields as _
                        #(self.#filters as _),* // Example: fields as _
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
                pub async fn update(&self, conn: &mut sqlx::PgConnection) -> Result<sqlx::any::AnyQueryResult, sqlx::Error>
                {
                    let query_result = sqlx::query!(
                        #query,
                        #(self.#columns as _),*, // Example: fields as _
                        #(self.#filters as _),* // Example: fields as _
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
