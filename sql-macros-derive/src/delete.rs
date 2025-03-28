use crate::parser::{
    Table, extract_fields_as_params, fields_named_struct, generate_sql_params_condition,
    get_filters, get_method_params, parse_fields_with_type,
};
use proc_macro::TokenStream;
use quote::quote;

fn generate_method(
    method_name: &str,
    _struct_name: &proc_macro2::Ident,
    params: &proc_macro2::TokenStream,
    query: &str,
    filter_fields: &Vec<proc_macro2::Ident>,
) -> proc_macro2::TokenStream {
    let mn =
        syn::parse_str::<proc_macro2::Ident>(method_name).expect("Failed to parse code string");
    quote! {
        #[doc=#query]
        pub async fn #mn(conn: &mut sqlx::PgConnection, #params) -> Result<sqlx::any::AnyQueryResult, sqlx::Error> {
            let result = sqlx::query!(
                #query,
                #(#filter_fields),*
            )
            .execute(&mut *conn)
            .await?;
            Ok(result.into()) // .into need for different db postgres and mysql
        }
    }
}

pub fn sql_delete_macro_derive(input: &mut syn::DeriveInput) -> TokenStream {
    let table = Table::parse(input);
    let struct_name = input.ident.clone();
    let table_name = table.get_name();

    let fields = fields_named_struct(input);
    let fields_with_type = parse_fields_with_type(fields, "delete");

    let mut methods = vec![];
    for field_with_type in fields_with_type {
        let params = get_method_params(vec![field_with_type.clone()]);
        let filter_fields = get_filters(vec![field_with_type.clone()]);
        let sql_filters = generate_sql_params_condition(&filter_fields);

        let query = format!("DELETE FROM {table_name} WHERE {sql_filters}");
        methods.push(generate_method(
            &format!("delete_by_{}", field_with_type.0),
            &struct_name,
            &params,
            &query,
            &filter_fields,
        ));
    }

    let ff = extract_fields_as_params(fields);
    for (method_name, method_fields) in table.get_delete() {
        let fields_with_type: Vec<(proc_macro2::Ident, syn::Type)> = ff
            .clone()
            .into_iter()
            .filter(|(name_field, _)| method_fields.contains(&name_field.to_string()))
            .collect();
        if fields_with_type.is_empty() {
            panic!(
                "No has params for method {method_name} or fields ({}) not contains in {struct_name}.",
                method_fields.join(", ")
            )
        }
        let params = get_method_params(fields_with_type.clone());
        let filter_fields = get_filters(fields_with_type);
        let sql_filters = generate_sql_params_condition(&filter_fields);
        let query = format!("DELETE FROM {table_name} WHERE {sql_filters}");

        methods.push(generate_method(
            &method_name,
            &struct_name,
            &params,
            &query,
            &filter_fields,
        ));
    }

    let token_stream = quote! {
        impl #struct_name {
            #(#methods)*
        }
    };
    token_stream.into()
}
