#[derive(Clone, Copy, PartialEq)]
pub enum ModuleType {
    Store,
    Map,
}

/// Configuration options parsed from macro attributes.
#[derive(Clone, Copy, Default)]
pub struct HandlerOptions {
    /// When true, skip calling `substreams::skip_empty_output()`.
    pub keep_empty_output: bool,
    /// When true, disable generation of the testable `__impl_<name>` function.
    /// By default (false), the macro generates both the testable function and the WASM export.
    pub no_testable: bool,
    /// When true, use quick-protobuf instead of prost for encoding/decoding.
    /// Requires the `quick-protobuf` feature to be enabled on the substreams crate.
    pub quick_protobuf: bool,
    /// When true, use buffa instead of prost for encoding/decoding.
    /// Requires the `buffa` feature to be enabled on the substreams crate.
    ///
    /// This selects buffa's OWNED api. buffa's own docs rate that path at
    /// "within roughly +/-10% of prost"; the fast paths are the borrowed ones,
    /// selected with `buffa_lazy` below.
    pub buffa: bool,
    /// When true, decode with buffa's LAZY VIEW -- one non-recursive scan that
    /// records nested/repeated message fields as undecoded byte ranges, decoded
    /// on access. This is buffa's fastest decode path (measured 2.9-4.9x prost
    /// natively, 2.5-14x in wasm fuel, versus 1.4x for the owned api).
    ///
    /// It changes the handler signature: the view borrows from the input
    /// buffer, so the handler takes `&FooLazyView<'_>` rather than an owned
    /// `Foo`. The generated export keeps the buffer alive for the whole call,
    /// so the borrow is valid for the handler's entire body.
    pub buffa_lazy: bool,
}

impl HandlerOptions {
    /// Parse options from a comma-separated attribute string.
    /// Supported options: `no_testable`, `keep_empty_output`, `quick_protobuf`
    ///
    /// Examples:
    /// - `""` -> defaults (testable enabled, prost)
    /// - `"no_testable"` -> disable testable function generation
    /// - `"keep_empty_output"` -> keep_empty_output = true
    /// - `"quick_protobuf"` -> use quick-protobuf instead of prost
    /// - `"buffa"` -> use buffa's owned api instead of prost
    /// - `"buffa_lazy"` -> use buffa's lazy view (handler takes &FooLazyView<'_>)
    /// - `"no_testable, keep_empty_output"` -> both options set
    pub fn parse(args: &str) -> Result<Self, String> {
        let mut options = Self::default();

        if args.is_empty() {
            return Ok(options);
        }

        for part in args.split(',') {
            match part.trim() {
                "no_testable" => options.no_testable = true,
                "keep_empty_output" => options.keep_empty_output = true,
                "quick_protobuf" => options.quick_protobuf = true,
                "buffa" => options.buffa = true,
                "buffa_lazy" => options.buffa_lazy = true,
                other => {
                    return Err(format!(
                        "Unknown option '{}'. Valid options are: no_testable, keep_empty_output, quick_protobuf, buffa, buffa_lazy",
                        other
                    ))
                }
            }
        }

        Ok(options)
    }
}

pub struct FinalConfiguration {
    pub module_type: ModuleType,
}

// struct Configuration {
//     module_type: Option<ModuleType>
// }
//
// impl Configuration {
//     fn new() -> Self {
//         Configuration {
//             module_type: None,
//         }
//     }
//
//     fn set_module_type(&mut self, runtime: syn::Lit, span: Span) -> Result<(), syn::Error> {
//         if self.module_type.is_some() {
//             return Err(syn::Error::new(span, "`type` set multiple times."));
//         }
//
//         let runtime_str = parse_string(runtime, span, "type")?;
//         let mod_type =
//             ModuleType::from_str(&runtime_str).map_err(|err| syn::Error::new(span, err))?;
//         self.module_type = Some(mod_type);
//         Ok(())
//     }
//
//     fn build(&self) -> Result<FinalConfiguration, syn::Error> {
//         let mod_type = self.module_type.unwrap_or(ModuleType::Map);
//         Ok(FinalConfiguration {
//             module_type: mod_type,
//         })
//     }
// }

// fn parse_string(int: syn::Lit, span: Span, field: &str) -> Result<String, syn::Error> {
//     match int {
//         syn::Lit::Str(s) => Ok(s.value()),
//         syn::Lit::Verbatim(s) => Ok(s.to_string()),
//         _ => Err(syn::Error::new(
//             span,
//             format!("Failed to parse value of `{}` as string.", field),
//         )),
//     }
// }

// fn build_config(
//     args: AttributeArgs,
// ) -> Result<FinalConfiguration, syn::Error> {
//
//     let mut config = Configuration::new();
//
//     for arg in args {
//         match arg {
//             syn::NestedMeta::Meta(syn::Meta::NameValue(namevalue)) => {
//                 let ident = namevalue
//                     .path
//                     .get_ident()
//                     .ok_or_else(|| {
//                         syn::Error::new_spanned(&namevalue, "Must have specified ident")
//                     })?
//                     .to_string()
//                     .to_lowercase();
//                 match ident.as_str() {
//                     "type" => {
//                         config.set_module_type(
//                             namevalue.lit.clone(),
//                             syn::spanned::Spanned::span(&namevalue.lit),
//                         )?;
//                     }
//                     name => {
//                         let msg = format!(
//                             "Unknown attribute {} is specified; expected one of: `type`",
//                             name,
//                         );
//                         return Err(syn::Error::new_spanned(namevalue, msg));
//                     }
//                 }
//             }
//             other => {
//                 return Err(syn::Error::new_spanned(
//                     other,
//                     "Unknown attribute inside the macro",
//                 ));
//             }
//         }
//     }
//     config.build()
// }
