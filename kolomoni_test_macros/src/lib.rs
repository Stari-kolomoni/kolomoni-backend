use proc_macro_error2::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::{ItemFn, Stmt, parse_quote};



#[proc_macro_error]
#[proc_macro_attribute]
pub fn test(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item = match syn::parse::<ItemFn>(item) {
        Ok(fn_item) => fn_item,
        Err(_) => {
            abort_call_site!(
                "The kolomoni_test_macros::test attribute macro only supports functions (tests)."
            );
        }
    };

    // Prepend global test mutex lock guard acquisition
    // and append global test mutex drop.

    let mutex_acquisition: Stmt = parse_quote! {
        let __ktm_test_mutex = kolomoni_test_core::_private::GLOBAL_TEST_MUTEX.lock().unwrap();
    };

    let mutex_drop: Stmt = parse_quote! {
        drop(__ktm_test_mutex);
    };

    item.block.stmts.insert(0, mutex_acquisition);
    item.block.stmts.push(mutex_drop);

    let output_item = quote! {
        #[allow(clippy::await_holding_lock)]
        #[::tokio::test]
        #item
    };

    proc_macro::TokenStream::from(output_item)
}
