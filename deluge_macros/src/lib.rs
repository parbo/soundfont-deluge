use heck::MixedCase;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DataEnum, DeriveInput};

#[proc_macro_derive(serde_enum)]
pub fn serde_enum(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let variants: Vec<_> = match data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let vs = variants.into_iter().map(|v| v.ident).collect();
            vs
        }
        _ => panic!(), // TODO: emit nice error
    };
    let variants_str: Vec<_> = variants
        .iter()
        .map(|ident| {
            let _ = &ident;
	    // Special case for FX
            return ident.to_string().to_mixed_case().replace("Fx", "FX");
        })
        .collect();

    //println!("{:?}", variants_str);

    let output = quote! {
            impl Serialize for #ident {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                   where S: Serializer,
                {
                  match *self {
        #(
            #ident::#variants => serializer.serialize_str(#variants_str)
        ),*
            }
                }
            }

        impl<'de> Deserialize<'de> for #ident {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
            D: Deserializer<'de>,
                {
            let s = String::deserialize(deserializer)?;
            match s.as_str() {
            #(
            #variants_str => Ok(#ident::#variants)
            ),*,
            other => Err(serde::de::Error::custom(other.to_string()))
            }
                }
            }
    };

    output.into()
}
