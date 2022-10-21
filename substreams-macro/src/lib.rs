use proc_macro::TokenStream;

mod config;
mod errors;
mod handler;
mod store;

#[proc_macro_attribute]
pub fn map(args: TokenStream, item: TokenStream) -> TokenStream {
    return handler::main(args, item, config::ModuleType::Map);
}
#[proc_macro_attribute]
pub fn map_trait(args: TokenStream, item: TokenStream) -> TokenStream {
    return handler::main_treat(args, item, config::ModuleType::Map);
}

#[proc_macro_attribute]
pub fn map_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    return handler::main_impl(args, item, config::ModuleType::Map);
}

#[proc_macro_attribute]
pub fn store(args: TokenStream, item: TokenStream) -> TokenStream {
    return handler::main(args, item, config::ModuleType::Store);
}

// todo: remove this once satisfied with implementation of StoreDelete
#[proc_macro_derive(StoreWriter)]
pub fn derive(input: TokenStream) -> TokenStream {
    store::main(input)
}
