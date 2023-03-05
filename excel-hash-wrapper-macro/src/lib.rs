use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs};

fn get_nested_meta_name(nested_meta: &syn::NestedMeta) -> String {
    match nested_meta {
        syn::NestedMeta::Meta(meta) => match meta {
            syn::Meta::Path(ident) => ident.get_ident().cloned().unwrap().to_string(),
            _ => panic!("Unsupported macro argument"),
        },
        syn::NestedMeta::Lit(_) => panic!("Only identifiers are supported as an arguments to the macro"),
    }
}

#[proc_macro_attribute]
pub fn excel_hash_wrapper(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let args = args.clone().into_iter().map(|a| get_nested_meta_name(&a));

    // Generate all "X_hash", "X_hash_pre" and "X_hash_suffix" strings as well as getter names
    let args_hash = args.clone().into_iter().map(|a| format!("{}_hash", a));
    let args_hash_pre = args.clone().into_iter().map(|a| format!("{}_hash_pre", a));
    let args_hash_suffix = args.clone().into_iter().map(|a| format!("{}_hash_suffix", a));
    let get_args_hash = args.clone().into_iter().map(|a| format!("get_{}_hash", a));

    let args_hash: Vec<proc_macro2::TokenStream> = args_hash.map(|a| a.parse().unwrap()).collect();
    let args_hash_pre: Vec<proc_macro2::TokenStream> = args_hash_pre.map(|a| a.parse().unwrap()).collect();
    let args_hash_suffix: Vec<proc_macro2::TokenStream> = args_hash_suffix.map(|a| a.parse().unwrap()).collect();
    let get_args_hash: Vec<proc_macro2::TokenStream> = get_args_hash.map(|a| a.parse().unwrap()).collect();

    let mut found_struct = false;
    let mut struct_name = None;

    let modified_struct: proc_macro::TokenStream = input.into_iter().map(|r| {
        match &r {
            &proc_macro::TokenTree::Ident(ref ident) if ident.to_string() == "struct" => { // react on keyword "struct" so we don't randomly modify non-structs
                found_struct = true;
                r
            },
            &proc_macro::TokenTree::Ident(ref ident) if found_struct == true && struct_name == None => { // Next ident right after "struct" is the struct name
                struct_name = Some(ident.to_string());
                r
            },
            &proc_macro::TokenTree::Group(ref group) if group.delimiter() == proc_macro::Delimiter::Brace && found_struct == true => { // Opening brackets for the struct
                let mut stream = proc_macro::TokenStream::new();

                stream.extend(
                    // For each hash name, generate three fields: prefix, suffix and the one that combines them
                    vec![proc_macro::TokenStream::from(quote!(
                        #(
                            pub #args_hash_pre: Option<u8>,
                            pub #args_hash_suffix: Option<u32>,
                            pub #args_hash: Option<u64>,
                        )*
                    ))]
                );
                stream.extend(group.stream());

                proc_macro::TokenTree::Group(
                    proc_macro::Group::new(
                        proc_macro::Delimiter::Brace,
                        stream
                    )
                )
            }
            _ => r
        }
    }).collect();

    let struct_name: proc_macro2::TokenStream = match &struct_name {
        None => panic!("Failed to find struct name"),
        Some(name) => name.clone().parse().unwrap(),
    };

    let implementation = vec![proc_macro::TokenStream::from(quote!(
        // This implements getters for hashes
        // They first try to extract the value from u64 field
        // If it fails, they default to combining prefix and suffix
        impl #struct_name {
            #(
                pub fn #get_args_hash (&self) -> u64 {
                    match self.#args_hash {
                        Some(value) => value,
                        None => match self.#args_hash_pre {
                            None => panic!("Attempt to get an empty prefix for {} (object {:?})", stringify!(#args_hash), self),
                            Some(prefix) => match self.#args_hash_suffix {
                                None => panic!("Attempt to get an empty suffix for {} (object {:?})", stringify!(#args_hash), self),
                                Some(suffix) => IdManager::get_hash_by_prefix_suffix(prefix, suffix),
                            }
                        }
                    }
                }
            )*
        }

    ))];

    let mut ret = proc_macro::TokenStream::new();
    ret.extend(modified_struct);
    ret.extend(implementation);
    println!("{}", ret);
    ret
}