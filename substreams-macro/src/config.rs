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
}

impl HandlerOptions {
    /// Parse options from a comma-separated attribute string.
    /// Supported options: `no_testable`, `keep_empty_output`
    ///
    /// Examples:
    /// - `""` -> defaults (testable enabled)
    /// - `"no_testable"` -> disable testable function generation
    /// - `"keep_empty_output"` -> keep_empty_output = true
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
                other => {
                    return Err(format!(
                        "Unknown option '{}'. Valid options are: no_testable, keep_empty_output",
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
