use darling::FromAttributes;
use inflector::Inflector;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{DeriveInput, LitStr};

use crate::attrs::*;

macro_rules! syn_err {
  ($span:expr, $message:expr) => {
    return Err(syn::Error::new($span, $message))
  };

  ($message:expr) => {
    syn_err!(Span::call_site(), $message)
  };
}

fn fmt_has_prefix(fmt: &str) -> bool {
  fmt.contains("{}")
}

// this is a somehow hacky way to find if a type is Option
// it matches [::]core::option::Option, [::]std::option::Option and Option
// this is needed because we can't implement FromStr for Option<T: FromStr>
// nevertheless we use the metre::UnOption::T associated type
// so if this gives us a false posistive match it will fail to compile
// it will also fail to compile for a false negative match for a type that
// doesn't implement FromStr -> Option<T>
fn ty_is_option(ty: &syn::Type) -> bool {
  fn extract_option_segment(path: &syn::Path) -> Option<&syn::PathSegment> {
    let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
      acc.push_str(&v.ident.to_string());
      acc.push('.');
      acc
    });

    ["Option.", "std.option.Option.", "core.option.Option."]
      .into_iter()
      .find(|s| idents_of_path == *s)
      .and_then(|_| path.segments.last())
  }

  fn extract_type_path(ty: &syn::Type) -> Option<&syn::Path> {
    match *ty {
      syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
      _ => None,
    }
  }

  match extract_type_path(ty) {
    None => false,
    Some(path) => extract_option_segment(path).is_some(),
  }
}

pub fn config(input: DeriveInput) -> Result<TokenStream, syn::Error> {
  //let generics = &input.generics;
  let generics = &input.generics;
  let name = &input.ident;
  let vis = &input.vis;
  let container_attrs = ContainerAttrs::from_attributes(&input.attrs)?;

  let metre = container_attrs
    .metre_crate
    .clone()
    .map(|path| quote! { #path })
    .unwrap_or_else(|| quote! { ::metre });

  let partial_name = container_attrs
    .partial_name
    .clone()
    .unwrap_or_else(|| syn::Ident::new(&format!("Partial{}", name), Span::call_site()));

  let rename_all = container_attrs.rename_all_inflection()?;
  let rename_all_serde_attr = rename_all.map(|_| {
    let spanned = container_attrs.rename_all.as_ref().unwrap();
    let lit = LitStr::new(spanned, spanned.span());
    quote! { #[serde(rename_all = #lit)] }
  });

  let deny_unknown_attr = if container_attrs.allow_unknown_fields {
    quote! {}
  } else {
    quote! { #[serde(deny_unknown_fields)] }
  };

  if *container_attrs.skip_env {
    if let Some(env_prefix) = container_attrs.env_prefix {
      syn_err!(
        env_prefix.span(),
        "cannot use both env_prefix and skip_env in the same item"
      )
    }
  };

  let container_env_prefix_fmt: LitStr = container_attrs
    .env_prefix
    .map(|v| LitStr::new(&v, v.span()))
    .unwrap_or_else(|| LitStr::new("{}", Span::call_site()));

  let get_container_env_prefix = if fmt_has_prefix(&container_env_prefix_fmt.value()) {
    quote! { format!(#container_env_prefix_fmt, env_prefix) }
  } else {
    quote! { String::from(#container_env_prefix_fmt) }
  };

  let item = match &input.data {
    syn::Data::Enum(_) => syn_err!("enums are not yet supported"),
    syn::Data::Union(_) => syn_err!("unions not supported"),
    syn::Data::Struct(item) => item,
  };

  let fields = match &item.fields {
    syn::Fields::Unit => syn_err!("unit structs are not supported"),
    syn::Fields::Unnamed(_) => syn_err!("tuple structs are not supported"),
    syn::Fields::Named(named) => named,
  };

  let mut partial_fields_declaration = Vec::<TokenStream>::new();
  let mut destructure_fields = Vec::<TokenStream>::new();
  let mut merge_partial_fields = Vec::<TokenStream>::new();
  let mut from_env_fields = Vec::<TokenStream>::new();
  let mut missing_fields_stmts = Vec::<TokenStream>::new();
  let mut is_empty_stmts = Vec::<TokenStream>::new();
  let mut from_partial_fields = Vec::<TokenStream>::new();
  let mut default_fields = Vec::<TokenStream>::new();

  for field in &fields.named {
    let vis = &field.vis;
    let ident = field.ident.clone().unwrap();
    let ty = &field.ty;
    let span = ident.span();
    let is_option = ty_is_option(ty);

    macro_rules! span_quote {
      ($($tt:tt)*) => {
        quote_spanned!(span => $($tt)*)
      }
    }

    let attrs = FieldArgs::from_attributes(&field.attrs)?;

    let field_name = match &attrs.rename {
      Some(name) => Ident::new(name, span),
      None => match rename_all {
        Some(inflection) => Ident::new(&inflection.apply(&ident.to_string()), ident.span()),
        None => ident.clone(),
      },
    };

    let serde_partial_rename_attr = match &attrs.rename {
      None => quote! {},
      Some(name) => span_quote! { #[serde(rename = #name)] },
    };

    let serde_flatten_attr = match attrs.flatten {
      false => quote! {},
      true => quote! { #[serde(flatten)] },
    };

    let env_name = match &attrs.rename {
      Some(name) => name.to_string().to_screaming_snake_case(),
      None => ident.to_string().to_screaming_snake_case(),
    };

    let env_fmt: LitStr = attrs
      .env
      .as_ref()
      .map(|v| LitStr::new(v, v.span()))
      .unwrap_or_else(|| {
        if attrs.flatten {
          LitStr::new("{}", span)
        } else {
          LitStr::new(&format!("{{}}{}", env_name), span)
        }
      });

    let get_field_env_key = if fmt_has_prefix(&env_fmt.value()) {
      span_quote! { format!(#env_fmt, container_env_prefix) }
    } else {
      span_quote! { #env_fmt.to_string() }
    };

    match attrs.default {
      None => {
        if attrs.nested {
          default_fields.push(quote! {
            #ident: <#ty as #metre::Config>::Partial::defaults(),
          })
        } else {
          default_fields.push(quote! {
            #ident: ::core::option::Option::None,
          })
        }
      }

      Some(expr) => {
        default_fields.push(quote! {
          #ident: ::core::option::Option::Some(#expr),
        });
      }
    };

    let partial_ty: TokenStream;
    let mut merge_fn: TokenStream;
    let mut merge_map_err: TokenStream;

    let field_name_str = field_name.to_string();

    match attrs.nested {
      false => {
        partial_ty = span_quote! { ::core::option::Option<#ty> };
        merge_fn = span_quote! { #metre::util::merge_flat };
        merge_map_err = quote! {};
      }

      true => {
        partial_ty = span_quote! { <#ty as #metre::Config>::Partial };
        merge_fn = span_quote! { #metre::util::merge_nested };
        merge_map_err = quote! {
          .map_err(|e| {
            #metre::error::MergeError {
              field: format!("{}.{}", #field_name_str, e.field),
              message: e.message
            }
          })
        };
      }
    };

    if let Some(merge) = attrs.merge {
      merge_fn = quote! { #merge };
      merge_map_err = span_quote! {
        .map_err(|e| {
          #metre::error::MergeError {
            field: String::from(#field_name_str),
            message: e.to_string()
          }
        })
      }
    }

    if *attrs.skip_env {
      if let Some(env) = attrs.env {
        syn_err!(
          env.span(),
          "cannot use both env and skip_env in the same field"
        );
      }
    };

    let parse_env_fn = match &attrs.parse_env {
      None => {
        if is_option {
          span_quote! { <<#ty as #metre::util::UnOption>::T as ::std::str::FromStr>::from_str(&env_value).map(|v| ::core::option::Option::Some(::core::option::Option::Some(v))) }
        } else {
          span_quote! { <#ty as ::std::str::FromStr>::from_str(&env_value).map(::core::option::Option::Some) }
        }
      }
      Some(path) => {
        if is_option {
          span_quote! { #path(&env_value).map(::core::option::Option::Some) }
        } else {
          span_quote! { #path(&env_value) }
        }
      }
    };

    let serde_skip_serializing_if = if attrs.nested {
      let path = format!("{}::PartialConfig::is_empty", metre);
      span_quote! { #[serde(skip_serializing_if = #path)] }
    } else {
      span_quote! { #[serde(skip_serializing_if = "::core::option::Option::is_none")] }
    };

    partial_fields_declaration.push(span_quote! {
      #[serde(default)]
      #serde_skip_serializing_if
      #serde_partial_rename_attr
      #serde_flatten_attr
      #vis #ident: #partial_ty,
    });

    destructure_fields.push(span_quote! {#ident,});

    merge_partial_fields.push(span_quote! {
      #merge_fn(&mut self.#ident, #ident)#merge_map_err?;
    });

    if attrs.nested {
      missing_fields_stmts.push(span_quote! {
        for prop in #metre::PartialConfig::list_missing_properties(&self.#ident) {
          missing_fields.push(format!("{}.{}", #field_name_str, prop));
        };
      });

      is_empty_stmts.push(span_quote! {
        if !#metre::PartialConfig::is_empty(&self.#ident) {
          return false;
        };
      });

      from_partial_fields.push(span_quote! {
        #ident: #metre::Config::from_partial(#ident).unwrap(),
      });
    } else {
      if !is_option {
        missing_fields_stmts.push(span_quote! {
          if ::core::option::Option::is_none(&self.#ident) {
            missing_fields.push(String::from(#field_name_str));
          };
        });
      }

      is_empty_stmts.push(span_quote! {
        if !::core::option::Option::is_none(&self.#ident) {
          return false;
        };
      });

      if !is_option {
        from_partial_fields.push(span_quote! {
          #ident: ::core::option::Option::unwrap(#ident),
        });
      } else {
        from_partial_fields.push(span_quote! {
          #ident: #ident.unwrap_or(::core::option::Option::None),
        })
      }
    }

    let field_name_lit = LitStr::new(&field_name.to_string(), field_name.span());

    let skip_env = {
      if *container_attrs.skip_env {
        attrs.env.is_none()
      } else {
        *attrs.skip_env
      }
    };

    let from_env_field: TokenStream;

    if skip_env {
      from_env_field = span_quote! { #ident: ::core::option::Option::None, }
    } else if attrs.nested {
      from_env_field = span_quote! {
        #ident: {

          let mut nested_prefix: String = #get_field_env_key;
          if !nested_prefix.is_empty() && !nested_prefix.ends_with('_') {
            nested_prefix.push('_');
          }

          #metre::PartialConfig::from_env_with_provider_and_prefix(env, &nested_prefix).map_err(|e| {
            // set the correct deep path to the field
            #metre::error::FromEnvError {
              key: e.key,
              field: format!("{}.{}", #field_name_lit, e.field),
              message: e.message,
            }
          })?
        },
      };
    } else {
      // let map_from_env_value = if is_option {
      //   quote! { ::core::option::Option::Some(value) }
      // } else {
      //   quote! { value }
      // };

      from_env_field = span_quote! {
        #ident: {
          let key = #get_field_env_key;

          let env_string_option = env.get(&key).map_err(|e| {
            #metre::error::FromEnvError {
              key: key.clone(),
              field: String::from(#field_name_lit),
              message: e.to_string(),
            }
          })?;

         match env_string_option {
            None => ::core::option::Option::None,
            Some(env_value) => {
              #parse_env_fn.map_err(|e| {
                #metre::error::FromEnvError {
                  key,
                  field: String::from(#field_name_lit),
                  message: e.to_string(),
                }
              })?
            },
          }
        },
      }
    }

    from_env_fields.push(from_env_field);
  }

  let partial_struct_declaration = quote! {
    #[derive(::std::fmt::Debug, ::std::default::Default, ::serde::Serialize, ::serde::Deserialize)]
    #rename_all_serde_attr
    #deny_unknown_attr
    #vis struct #partial_name #generics {
      #(#partial_fields_declaration)*
    }
  };

  let partial_impl = quote! {
    impl #generics #metre::PartialConfig for #partial_name #generics {

      fn defaults() -> Self {
        Self {
          #(#default_fields)*
        }
      }

      fn merge(&mut self, other: Self) -> Result<(), #metre::error::MergeError> {
        let Self {
          #(#destructure_fields)*
        } = other;

        #(#merge_partial_fields)*

        Ok(())
      }

      fn from_env_with_provider_and_optional_prefix<E: #metre::EnvProvider>(env: &E, prefix: Option<&str>) -> Result<Self, #metre::error::FromEnvError> {

        let env_prefix = prefix.unwrap_or("");
        let container_env_prefix = #get_container_env_prefix;

        Ok(Self {
          #(#from_env_fields)*
        })
      }

      fn list_missing_properties(&self) -> Vec<String> {
        let mut missing_fields = vec![];
        #(#missing_fields_stmts)*
        missing_fields
      }

      fn is_empty(&self) -> bool {
        #(#is_empty_stmts)*
        true
      }
    }
  };

  let config_impl = quote! {
    impl #generics #metre::Config for #name #generics {
      type Partial = #partial_name #generics;
      fn from_partial(partial: Self::Partial) -> Result<Self, #metre::error::FromPartialError> {

        let missing_properties = #metre::PartialConfig::list_missing_properties(&partial);
        if !missing_properties.is_empty() {
          return Err(#metre::error::FromPartialError {
            missing_properties
          });
        }

        let Self::Partial {
          #(#destructure_fields)*
        } = partial;

        Ok(Self {
          #(#from_partial_fields)*
        })
      }
    }
  };

  let out = quote! {
    #config_impl

    #partial_struct_declaration

    #partial_impl

    impl #generics TryFrom<#partial_name #generics> for #name #generics {
      type Error = #metre::error::FromPartialError;
      #[inline(always)]
      fn try_from(partial: #partial_name #generics) -> Result<Self, Self::Error> {
          <#name #generics as #metre::Config>::from_partial(partial)
      }
    }
  };

  Ok(out)
}
