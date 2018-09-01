extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span};
use quote::TokenStreamExt;


#[proc_macro_derive(Address, attributes(address_repr))]
pub fn address(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_address(&ast);

    // Return the generated impl
    gen.into()
}

fn impl_address(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let ty = ast
        .attrs
        .iter()
        .find(|attr| {
            attr.path == syn::Path::from(Ident::new("address_repr", Span::call_site()))
        }).map(|attr| attr.tts.clone())
        .expect("#[derive(Address)] requires #[address_repr] attribute!");

    let mut token_stream: proc_macro2::TokenStream = {
        let repr = &ty;
        let tokens = quote! {
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
                    use ::hal9000::util::Align;
                    #name ( self.0.align_down(align) )
                }

                /// Align this address up to the provided alignment.
                fn align_up(&self, align: #repr) -> Self {
                    use ::hal9000::util::Align;
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
        };
        tokens.into()
    };
    let self_binops = [
        (quote!(Add), quote!(add)),
        (quote!(Sub), quote!(sub)),
        (quote!(Div), quote!(div)),
        (quote!(Mul), quote!(mul)),
        (quote!(Rem), quote!(rem)),
    ];
    for (op_ty, func) in &self_binops {
        let binop = impl_self_binop(quote!(::core::ops), name, op_ty, func);
        token_stream.extend(binop);
    }
    token_stream
}


fn impl_self_binop<A, B, C, D>(path: A, ty: B, op_ty: C, func: D) -> proc_macro2::TokenStream
where
    A: quote::ToTokens,
    B: quote::ToTokens,
    C: quote::ToTokens,
    D: quote::ToTokens,
{
    let mut tokens: proc_macro2::TokenStream = quote! {
        impl #path::#op_ty<#ty> for #ty {
            type Output = #ty;
            fn #func(self, rhs: #ty) -> Self::Output {
                #ty ( #path::#op_ty::#func(self.0, rhs.0) )
            }
        }
    }.into();
    tokens.append_all(&[
        // this is broken up into several `quote!`s so we don't hit
        // the recursion limit...
        quote! {
            impl<'a> #path::#op_ty<#ty> for &'a #ty {
                type Output = #ty;
                fn #func(self, rhs: #ty) -> Self::Output {
                    #ty ( #path::#op_ty::#func(self.0, rhs.0) )
                }
            }
        },
        quote! {
            impl<'a> #path::#op_ty<&'a #ty> for #ty {
                type Output = #ty;
                fn #func(self, rhs: &'a #ty) -> Self::Output {
                    #ty ( #path::#op_ty::#func(self.0, rhs.0) )
                }
            }
        },
        quote! {
            impl<'a> #path::#op_ty<&'a #ty> for &'a #ty {
                type Output = #ty;
                fn #func(self, rhs: &'a #ty) -> Self::Output {
                    #ty ( #path::#op_ty::#func(self.0, rhs.0) )
                }
            }
        },
    ]);
    tokens.into()
}
