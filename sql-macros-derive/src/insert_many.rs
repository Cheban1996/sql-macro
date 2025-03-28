use proc_macro::TokenStream;
use quote::quote;

use crate::parser::{Table, fields_named_struct, get_sql_columns};

pub fn sql_insert_many_macro_derive(input: &mut syn::DeriveInput) -> TokenStream {
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

    let query = format!(
        "INSERT INTO {table_name} ({sql_columns}) VALUES ({sql_column_index}) RETURNING *",
    );

    let token_stream = quote! {
        impl #struct_name {
            #[doc=#query]
            pub async fn insert<T>(&self, conn: &mut sqlx::PgConnection) -> Result<(), sqlx::Error>
            where
                T: Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            {
                let object = sqlx::query_as::<_, T>(#query)
                #(
                    .bind(&self.#idents)
                )*
                    .fetch_one(&mut *conn)
                    .await?;
                Ok(())
            }
        }
    };
    token_stream.into()
}
