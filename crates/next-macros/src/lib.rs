use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Marks a function as a server component.
///
/// Server components run only on the server and can access databases,
/// file systems, and other server-only resources directly.
///
/// ```rust,ignore
/// #[server_component]
/// fn article_list() -> Element {
///     div().child(h1().text("Articles"))
/// }
/// ```
#[proc_macro_attribute]
pub fn server_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;
    let block = &input.block;
    let output = &input.sig.output;

    let expanded = quote! {
        #vis fn #fn_name() #output {
            next_rs_rsc::global_registry().register_server(module_path!(), #fn_name_str);
            (|| #block)()
        }
    };

    expanded.into()
}

/// Marks a function as a client component.
///
/// Client components are shipped to the browser as WASM and can use
/// interactive features like event handlers and reactive state.
///
/// ```rust,ignore
/// #[client_component]
/// fn counter() -> Element {
///     let (count, set_count) = create_signal(0);
///     div().child(button().text("+").on_click(move |_| set_count.update(|n| *n += 1)))
/// }
/// ```
#[proc_macro_attribute]
pub fn client_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;
    let block = &input.block;
    let output = &input.sig.output;

    let expanded = quote! {
        #vis fn #fn_name() #output {
            next_rs_rsc::global_registry().register_client(module_path!(), #fn_name_str);
            (|| #block)()
        }
    };

    expanded.into()
}

/// Marks an async function as a server action.
///
/// Server actions can be called from client components and are
/// automatically serialized/deserialized across the network boundary.
///
/// ```rust,ignore
/// #[server_action]
/// async fn create_todo(title: String) -> Result<Todo, ActionError> {
///     db::insert_todo(&title).await
/// }
/// ```
#[proc_macro_attribute]
pub fn server_action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let expanded = quote! {
        #vis #sig {
            next_rs_rsc::global_registry().register_server(module_path!(), #fn_name_str);
            #block
        }
    };

    expanded.into()
}
