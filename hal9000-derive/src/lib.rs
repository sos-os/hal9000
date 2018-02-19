extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;


#[proc_macro_derive(Address, attributes(address_repr))]
pub fn address(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_address(&ast);

    // Return the generated impl
    gen.into()
}

fn impl_address(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let ty = ast.attrs.iter()
        .find(|attr|
            attr.path == syn::Path::from(syn::Ident::from("address_repr"))
        )
        .map(|attr| attr.tts.clone())
        .expect("#[derive(Address)] requires #[address_repr] attribute!");
    let repr = &ty;
    quote! {

        impl ::core::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                  -> ::core::fmt::Result {
                write!(f, "{}({:#08x})", stringify!(#name), self.0)
            }
        }

        impl Address for #name {
            type Repr = #repr;

            /// Align this address down to the provided alignment.
            fn align_down(&self, align: #repr) -> Self {
                use util::Align;
                #name ( self.0.align_down(align) )
            }

            /// Align this address up to the provided alignment.
            fn align_up(&self, align: #repr) -> Self {
                #name ( self.0.align_up(align) )
            }

            /// Returns true if this address is aligned on a page boundary.
            fn is_page_aligned<P: Page>(&self) -> bool {
                self.0 % P::SIZE as #repr == 0 as #repr
            }
        }

        impl ::core::convert::Into<#repr> for #name {
            fn into(self) -> #repr {
                self.0
            }
        }

        impl ::core::convert::From<#repr> for #name {
            fn from(r: #repr) -> #name {
                #name(r)
            }
        }
    }
}
