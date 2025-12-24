//! Procedural macros for navign
//!
//! This crate provides derive macros to reduce code duplication,
//! particularly for SQL repository implementations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, parse_macro_input};

/// Derive macro for generating SQL repository search and count methods
///
/// This macro generates the repetitive `search()` and `count()` methods
/// for types implementing IntRepository or UuidRepository traits.
///
/// # Attributes
///
/// - `#[repo(table = "table_name")]` - Specify the database table name
/// - `#[repo(search_fields = "field1, field2, field3")]` - Fields to search in
/// - `#[repo(select_fields = "id, name, ...")]` - Fields to select
/// - `#[repo(has_entity = true)]` - Whether this repository filters by entity_id
/// - `#[repo(default_sort = "created_at")]` - Default sort field
///
/// # Example
///
/// ```ignore
/// #[derive(SqlRepository)]
/// #[repo(
///     table = "areas",
///     search_fields = "name, description, beacon_code",
///     select_fields = "id, entity_id, name, description, floor_type, floor_name, beacon_code, polygon, created_at, updated_at",
///     has_entity = true,
///     default_sort = "created_at"
/// )]
/// pub struct Area {
///     // ... fields
/// }
/// ```
#[proc_macro_derive(SqlRepository, attributes(repo))]
pub fn derive_sql_repository(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract attributes
    let table_name = extract_attr(&input.attrs, "table")
        .unwrap_or_else(|| panic!("#[repo(table = \"...\")] attribute is required"));

    let search_fields = extract_attr(&input.attrs, "search_fields")
        .unwrap_or_else(|| panic!("#[repo(search_fields = \"...\")] attribute is required"));

    let select_fields = extract_attr(&input.attrs, "select_fields")
        .unwrap_or_else(|| panic!("#[repo(select_fields = \"...\")] attribute is required"));

    let has_entity = extract_attr(&input.attrs, "has_entity")
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(true);

    let default_sort =
        extract_attr(&input.attrs, "default_sort").unwrap_or_else(|| "created_at".to_string());

    let name = &input.ident;
    let search_field_list: Vec<&str> = search_fields.split(',').map(|s| s.trim()).collect();

    // Generate PostgreSQL search implementation
    let postgres_impl = if has_entity {
        generate_postgres_with_entity(
            name,
            &table_name,
            &select_fields,
            &search_field_list,
            &default_sort,
        )
    } else {
        generate_postgres_without_entity(
            name,
            &table_name,
            &select_fields,
            &search_field_list,
            &default_sort,
        )
    };

    // Generate SQLite search implementation
    let sqlite_impl = if has_entity {
        generate_sqlite_with_entity(
            name,
            &table_name,
            &select_fields,
            &search_field_list,
            &default_sort,
        )
    } else {
        generate_sqlite_without_entity(
            name,
            &table_name,
            &select_fields,
            &search_field_list,
            &default_sort,
        )
    };

    let expanded = quote! {
        #postgres_impl
        #sqlite_impl
    };

    TokenStream::from(expanded)
}

fn extract_attr(_attrs: &[Attribute], _name: &str) -> Option<String> {
    // TODO: Implement proper attribute parsing
    // For now, return None as this is a skeleton implementation
    None
}

fn generate_postgres_with_entity(
    name: &syn::Ident,
    table: &str,
    select_fields: &str,
    search_fields: &[&str],
    _default_sort: &str,
) -> proc_macro2::TokenStream {
    let like_conditions = search_fields
        .iter()
        .map(|field| format!("{} {} $2", field, "PLACEHOLDER"))
        .collect::<Vec<_>>()
        .join(" OR ");

    quote! {
        #[cfg(feature = "postgres")]
        impl #name {
            // Helper methods for search that can be used by the IntRepository implementation
            fn build_postgres_search_query(case_insensitive: bool, order_by: &str, direction: &str) -> String {
                let like_op = if case_insensitive { "ILIKE" } else { "LIKE" };
                format!(
                    r#"SELECT {} FROM {}
                       WHERE entity_id = $1 AND ({})
                       ORDER BY {} {}
                       LIMIT $3 OFFSET $4"#,
                    #select_fields,
                    #table,
                    #like_conditions.replace("PLACEHOLDER", "{}"),
                    order_by,
                    direction
                ).replace("{}", like_op)
            }

            fn build_postgres_count_query(case_insensitive: bool) -> String {
                let like_op = if case_insensitive { "ILIKE" } else { "LIKE" };
                format!(
                    r#"SELECT COUNT(*) as count FROM {}
                       WHERE entity_id = $1 AND ({})"#,
                    #table,
                    #like_conditions.replace("PLACEHOLDER", "{}"),
                ).replace("{}", like_op)
            }
        }
    }
}

fn generate_postgres_without_entity(
    _name: &syn::Ident,
    _table: &str,
    _select_fields: &str,
    _search_fields: &[&str],
    _default_sort: &str,
) -> proc_macro2::TokenStream {
    quote! {
        // PostgreSQL implementation without entity filter
        // (to be implemented similarly)
    }
}

fn generate_sqlite_with_entity(
    _name: &syn::Ident,
    _table: &str,
    _select_fields: &str,
    _search_fields: &[&str],
    _default_sort: &str,
) -> proc_macro2::TokenStream {
    quote! {
        // SQLite implementation with entity filter
        // (to be implemented similarly)
    }
}

fn generate_sqlite_without_entity(
    _name: &syn::Ident,
    _table: &str,
    _select_fields: &str,
    _search_fields: &[&str],
    _default_sort: &str,
) -> proc_macro2::TokenStream {
    quote! {
        // SQLite implementation without entity filter
        // (to be implemented similarly)
    }
}
