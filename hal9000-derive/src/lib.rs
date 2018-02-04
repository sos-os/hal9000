extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

macro_rules! impl_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl ::core::ops::$name<$ty> for $ty {
            type Output = $ty;

            #[inline] fn $fun(self, rhs: $ty) -> $ty {
                $ty(expr!(self.0 $op rhs.0))
            }
        }
        impl ::core::ops::$name<$size> for $ty {
            type Output = $ty;

            #[inline] fn $fun(self, rhs: $size) -> $ty {
                $ty(expr!(self.0 $op rhs))
            }
        }

        forward_ref_binop! {
            $name, $fun for $ty, $ty
        }
        forward_ref_binop! {
            $name, $fun for $ty, $size
        }
    )*}
}

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
    let ty = ast.attrs.get(0)
        .and_then(|attr| {
            // if attr.path == syn::Ident::from("address_repr").into() {
                let ty: syn::Type = syn::parse2(attr.tts.clone())
                    .expect("address_repr type");
                Some(ty)
            // } else {
            //     None
            // }
        })
        .expect("#[derive(Address)] requires #[address_repr] attribute!");
    let repr = &ty;
    quote! {

        impl ops::Deref for #name {
            type Target = #repr;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Address for #name {
            type Repr = #repr;
            /// Align this address down to the provided alignment.
            fn align_down(&self, align: Self::Repr) -> Self {
                use util::Align;
                #name ( self.0.align_down(align) )
            }

            /// Align this address up to the provided alignment.
            fn align_up(&self, align: Self::Repr) -> Self {
                #name ( self.0.align_up(align) )
            }

            /// Returns true if this address is aligned on a page boundary.
            fn is_page_aligned<P: Page>(&self) -> bool {
                **self % P::SIZE as #repr == 0 as #repr
            }
        }
    }
}