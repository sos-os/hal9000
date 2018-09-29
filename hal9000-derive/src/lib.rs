#![recursion_limit = "128"]
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span};
use quote::{TokenStreamExt};

#[proc_macro_derive(Address, attributes(address_repr))]
pub fn address(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Build the impl
    let mut gen = proc_macro2::TokenStream::new();
    gen.append_all(&[impl_address(&ast), impl_number(&ast)]);

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(Number)]
pub fn number(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_number(&ast);

    // Return the generated impl
    gen.into()
}

fn impl_address(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    fn get_repr(ast: &syn::DeriveInput) -> Option<syn::NestedMeta> {
        let attr = ast.attrs.iter().find(|attr| {
            attr.path
                == syn::Path::from(Ident::new(
                    "address_repr",
                    Span::call_site(),
                ))
        });
        let meta = attr?.clone().interpret_meta();
        if let syn::Meta::List(list) = meta? {
            let nested = list.nested.clone();
            if nested.len() == 1 {
                return nested
                    .first()
                    .map(syn::punctuated::Pair::into_value)
                    .cloned();
            }
        };
        None
    }

    let name = &ast.ident;
    let ty = get_repr(ast)
        .expect("#[derive(Address)] requires #[address_repr] attribute!");
    let repr = &ty;
    let mut tokens: proc_macro2::TokenStream = quote! {
        impl ::core::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                write!(f, "{}({:p})", stringify!(#name), self)
            }
        }
        impl ::core::fmt::Pointer for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                write!(f, "{:p}", self.as_ptr())
            }
        }
    };
    // this is broken up into several `quote!`s so we don't hit
    // the recursion limit...
    tokens.append_all(&[
        quote! {
            impl Address for #name {
                type Repr = #repr;

                /// Align this address down to the provided alignment.
                fn align_down(&self, align: usize) -> Self {
                    use ::hal9000::util::Align;
                    #name::from( self.0.align_down(align as #repr) )
                }

                /// Align this address up to the provided alignment.
                fn align_up(&self, align: usize) -> Self {
                    use ::hal9000::util::Align;
                    #name::from( self.0.align_up(align as #repr) )
                }

                /// Returns true if this address is aligned on a page boundary.
                fn is_page_aligned<P: Page>(&self) -> bool {
                    self.0 % P::SIZE as #repr == 0 as #repr
                }

                #[inline(always)]
                fn as_ptr<T>(&self) -> *const T {
                    self.0 as *const T
                }

                #[inline(always)]
                fn as_mut_ptr<T>(&self) -> *mut T {
                    self.0 as *mut T
                }
            }
        },
        quote! {
            impl ::core::convert::Into<#repr> for #name {
                fn into(self) -> #repr {
                    self.0
                }
            }
        },
        quote! {
            impl ::core::iter::Step for #name {
                fn steps_between(start: &Self, end: &Self) -> Option<usize> {
                    <#repr>::steps_between(&start.0, &end.0)
                }

                /// Replaces this step with `1`, returning itself
                fn replace_one(&mut self) -> Self {
                    self.0 = 1;
                    *self
                }

                /// Replaces this step with `0`, returning itself
                fn replace_zero(&mut self) -> Self {
                    self.0 = 0;
                    *self
                }

                /// Adds one to this step, returning the result
                fn add_one(&self) -> Self {
                    Self::from( self.0 + 1 )
                }

                /// Subtracts one to this step, returning the result
                fn sub_one(&self) -> Self {
                    Self::from( self.0 - 1 )
                }

                /// Add an usize, returning None on overflow
                fn add_usize(&self, n: usize) -> Option<Self> {
                    use ::core::#repr;
                    if n > (#repr::MAX as usize) {
                        None
                    } else {
                        Some(Self::from( self.0 + (n as #repr)))
                    }
                }
            }
        },
        // quote!(impl hal9000::util::Align for #name {})
    ]);

    // Only generate `From<usize>` impl when the repr is not usize.
    let usize_ty = syn::NestedMeta::from(
        syn::Meta::from(Ident::new("usize", Span::call_site()))
    );
    if repr != &usize_ty {
        tokens.extend(quote! {
            impl ::core::convert::From<usize> for #name {
                fn from(r: usize) -> #name {
                    #name::from(r as #repr)
                }
            }
        });
    }
    tokens
}

fn impl_number(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let mut tokens = proc_macro2::TokenStream::new();
    for (op_ty, func) in &[
        (quote!(Add), quote!(add)),
        (quote!(Sub), quote!(sub)),
        (quote!(Div), quote!(div)),
        (quote!(Mul), quote!(mul)),
        (quote!(Rem), quote!(rem)),
        (quote!(BitAnd), quote!(bitand)),
    ] {
        let binop = impl_self_binop(quote!(::core::ops), name, op_ty, func);
        tokens.extend(binop);
    }
    tokens.extend(impl_unary_op(
        quote!(::core::ops),
        name,
        quote!(Not),
        quote!(not),
    ));
    tokens
}

fn impl_self_binop<A, B, C, D>(
    path: A,
    ty: B,
    op_ty: C,
    func: D,
) -> proc_macro2::TokenStream
where
    A: quote::ToTokens,
    B: quote::ToTokens,
    C: quote::ToTokens,
    D: quote::ToTokens,
{
    let mut tokens = proc_macro2::TokenStream::new();
    // again, this helps us get around the recursion limit...
    tokens.append_all(&[
        quote! {
            impl #path::#op_ty<#ty> for #ty {
                type Output = #ty;
                fn #func(self, rhs: #ty) -> Self::Output {
                    #ty ( #path::#op_ty::#func(self.0, rhs.0) )
                }
            }
        },
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
    tokens
}

fn impl_unary_op<A, B, C, D>(
    path: A,
    ty: B,
    op_ty: C,
    func: D,
) -> proc_macro2::TokenStream
where
    A: quote::ToTokens,
    B: quote::ToTokens,
    C: quote::ToTokens,
    D: quote::ToTokens,
{
    let mut tokens = proc_macro2::TokenStream::new();
    // again, this helps us get around the recursion limit...
    tokens.append_all(&[
        quote! {
            impl #path::#op_ty for #ty {
                type Output = #ty;
                fn #func(self) -> Self::Output {
                    #ty ( #path::#op_ty::#func(self.0) )
                }
            }
        },
        quote! {
            impl<'a> #path::#op_ty for &'a #ty {
                type Output = #ty;
                fn #func(self) -> Self::Output {
                    #ty ( #path::#op_ty::#func(self.0) )
                }
            }
        },
    ]);
    tokens
}
