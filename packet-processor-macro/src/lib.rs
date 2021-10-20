extern crate regex;
extern crate convert_case;

use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs};
use regex::Regex;
use convert_case::{Case, Casing};

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
pub fn packet_processor(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    //let input = parse_macro_input!(input as DeriveInput);

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
                    vec![proc_macro::TokenStream::from(quote!(packet_callbacks: HashMap<proto::PacketId, fn(&mut Self, u32, &proto::PacketHead, Vec<u8>) -> ()>,))]
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

    let args = args.clone().into_iter().map(|a| get_nested_meta_name(&a));
    let re = Regex::new(r"Req$").unwrap();

    let request = args.clone().into_iter().filter(|a| a.ends_with("Req"));
    let notify = args.clone().into_iter().filter(|a| a.ends_with("Notify"));
    let response = request.clone().into_iter().map(|a| re.replace_all(&a, "Rsp").to_string());
    let req_handler = request.clone().into_iter().map(|a| format!("process_{}", &a[..a.len()-3].to_case(Case::Snake)));
    let notify_handler = notify.clone().into_iter().map(|a| format!("process_{}", &a[..a.len()-6].to_case(Case::Snake)));

    let request: Vec<proc_macro2::TokenStream> = request.map(|a| a.parse().unwrap()).collect();
    let notify: Vec<proc_macro2::TokenStream> = notify.map(|a| a.parse().unwrap()).collect();
    let response: Vec<proc_macro2::TokenStream> = response.map(|a| a.parse().unwrap()).collect();
    let req_handler: Vec<proc_macro2::TokenStream> = req_handler.map(|a| a.parse().unwrap()).collect();
    let notify_handler: Vec<proc_macro2::TokenStream> = notify_handler.map(|a| a.parse().unwrap()).collect();

    let struct_name: proc_macro2::TokenStream = match &struct_name {
        None => panic!("Failed to find struct name"),
        Some(name) => name.clone().parse().unwrap(),
    };

    let implementation = vec![proc_macro::TokenStream::from(quote!(
        impl PacketProcessor for #struct_name {
            fn register(&mut self) {
                let mut callbacks = &mut self.packet_callbacks;
                #(register_callback!(callbacks, #request, #response, #req_handler);)*
                #(register_callback!(callbacks, #notify, #notify_handler);)*
            }

            fn supported(&self) -> Vec<proto::PacketId> {
                return self.packet_callbacks.keys().cloned().collect();
            }

            fn process(&mut self, user_id: u32, packet_id: proto::PacketId, metadata: Vec<u8>, data: Vec<u8>) {
                let callback = self.packet_callbacks.get(&packet_id);
                let metadata = proto::PacketHead::decode(&mut std::io::Cursor::new(metadata)).unwrap();

                match callback {
                    Some(callback) => callback(self, user_id, &metadata, data),
                    None => println!("Unhandled packet {:?}", packet_id),
                }
            }
        }

    ))];

    let mut ret = proc_macro::TokenStream::new();
    ret.extend(modified_struct);
    ret.extend(implementation);
    //println!("{}", ret);
    ret
}
