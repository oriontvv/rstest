//! # Reuse `rstest`'s parametrized cases
//!
//! This crate give a way to define a tests set and apply them to every case you need to
//! test.
//!
//! With `rstest` crate you can define a tests list but if you want to apply the same tests
//! to another test function you must rewrite all cases or write some macros that do the job.
//! Both solutions have some drawbreak:
//!
//! - introduce duplication
//! - macros makes code harder to read and shift out the focus from tests core
//!
//! The aim of this crate is solve this problem. `rstest_resuse` expose two attributes:
//!
//! - `#[template]`: to define a template
//! - `#[apply]`: to apply a defined template to create tests
//!
//! Here is a simple example:
//!
//! ```
//! use rstest::rstest;
//! use rstest_reuse::{self, *};
//!
//! // Here we define the template. This define
//! // * The test list name to `two_simple_cases`
//! // * cases: here two cases that feed the `a`, `b` values
//! #[template]
//! #[rstest]
//! #[case(2, 2)]
//! #[case(4/2, 2)]
//! fn two_simple_cases(#[case] a: u32,#[case] b: u32) {}
//!
//! // Here we apply the `two_simple_cases` template: That is expanded in
//! // #[rstest]
//! // #[case(2, 2)]
//! // #[case(4/2, 2)]
//! // fn it_works(#[case] a: u32,#[case] b: u32) {
//! //     assert!(a == b);
//! // }
//! #[apply(two_simple_cases)]
//! fn it_works(a: u32, b: u32) {
//!     assert!(a == b);
//! }
//!
//!
//! // Here we reuse the `two_simple_cases` template to create two
//! // other tests
//! #[apply(two_simple_cases)]
//! fn it_fail(a: u32, b: u32) {
//!     assert!(a != b);
//! }
//! ```
//! If we run `cargo test` we have:
//!
//! ```text
//!     Finished test [unoptimized + debuginfo] target(s) in 0.05s
//!      Running target/debug/deps/playground-8a1212f8b5eb00ce
//!
//! running 4 tests
//! test it_fail::case_1 ... FAILED
//! test it_works::case_1 ... ok
//! test it_works::case_2 ... ok
//! test it_fail::case_2 ... FAILED
//!
//! failures:
//!
//! ---- it_fail::case_1 stdout ----
//! -------------- TEST START --------------
//! thread 'it_fail::case_1' panicked at 'assertion failed: a != b', src/main.rs:34:5
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//!
//! ---- it_fail::case_2 stdout ----
//! -------------- TEST START --------------
//! thread 'it_fail::case_2' panicked at 'assertion failed: a != b', src/main.rs:34:5
//!
//!
//! failures:
//!     it_fail::case_1
//!     it_fail::case_2
//!
//! test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
//!
//! error: test failed, to rerun pass '--bin playground'
//! ```
//!
//! Simple and neat!
//!
//! Note that if the test arguments names match the template's ones you can don't
//! repeate the arguments attributes.
//!
//! ## Composition and Values
//!
//! If you need to add some cases or values when apply a template you can leverage on
//! composition. Here a simple example:
//!
//! ```
//! use rstest::rstest;
//! use rstest_reuse::{self, *};
//!
//! #[template]
//! #[rstest]
//! #[case(2, 2)]
//! #[case(4/2, 2)]
//! fn base(#[case] a: u32, #[case] b: u32) {}
//!
//! // Here we add a new case and an argument in a value list:
//! #[apply(base)]
//! #[case(9/3, 3)]
//! fn it_works(a: u32, b: u32, #[values("a", "b")] t: &str) {
//!     assert!(a == b);
//!     assert!("abcd".contains(t))
//! }
//! ```
//!
//! `cargo test` runs 6 tests:
//!
//! ```text
//! running 6 tests
//! test it_works::case_1::t_2 ... ok
//! test it_works::case_2::t_2 ... ok
//! test it_works::case_2::t_1 ... ok
//! test it_works::case_3::t_2 ... ok
//! test it_works::case_3::t_1 ... ok
//! test it_works::case_1::t_1 ... ok
//! ```
//!
//! Template can also used for `#[values]` and `#[with]` arguments if you need:
//!
//! ```
//! use rstest::*;
//! use rstest_reuse::{self, *};
//!
//! #[template]
//! #[rstest]
//! fn base(#[with(42)] fix: u32, #[values(1,2,3)] v: u32) {}
//!
//! #[fixture]
//! fn fix(#[default(0)] inner: u32) -> u32 {
//!     inner
//! }
//!
//! #[apply(base)]
//! fn use_it_with_fixture(fix: u32, v: u32) {
//!     assert!(fix%v == 0);
//! }
//!
//! #[apply(base)]
//! fn use_it_without_fixture(v: u32) {
//!     assert!(24 % v == 0);
//! }
//! ```
//!
//! `cargo test` runs 6 tests:
//!
//! ```text
//! running 6 tests
//! test use_it_with_fixture::v_1 ... ok
//! test use_it_without_fixture::v_1 ... ok
//! test use_it_with_fixture::v_3 ... ok
//! test use_it_without_fixture::v_2 ... ok
//! test use_it_without_fixture::v_3 ... ok
//! test use_it_with_fixture::v_2 ... ok
//! ```
//!
//!
//! ## Cavelets
//!
//! ### `use rstest_resuse` at the top of your crate
//!
//! You **should** add `use rstest_resuse` at the top of your crate:
//!
//! ```
//! #[cfg(test)]
//! use rstest_reuse;
//! ```
//!
//! This is due `rstest_reuse::template` define a macro that need to call a `rstest_resuse`'s macro.
//! I hope to remove this in the future but for now we should live with it.
//!
//! Note that
//! ```
//! use rstest_reuse::*;
//! ```
//! is not enougth: this statment doesn't include `rstest_reuse` but just its public items.
//!
//!
//! ### Define `template` before `apply` it
//!
//! `template` attribute define a macro that `apply` will use. Macro in rust are expanded in
//! a single depth-first, lexical-order traversal of a crate’s source, that means the template
//! definition should be allways before the `apply`.
//!
//! ### Tag modules with `#[macro_use]`
//!
//! If you define a `template` in a module and you want to use it outside the module you should
//! _lift_ it by mark the module with the `#[macro_use]` attribute. This attribute make your
//! `template` visibe outside this module but not at the upper level. When a `template` is
//! defined you can use it in all submodules that follow the definition.
//!
//! If you plan to spread your templates in some modules and you use a unique name for each template
//! consider to add the global attribute `!#[macro_use]` at crate level: this put all your templates
//! available everywhere: you should
//! just take care that a `template` should be defined before the `apply` call.
//!
//!
//! ## Disclamer
//!
//! This crate is in developer stage. I don't know if I'll include it in `rstest` or changing some syntax in
//! the future.
//!
//! I did't test it in a lot of cases: if you have some cases where it doesn't works file a ticket on
//! [`rstest`](https://github.com/la10736/rstest)

extern crate proc_macro;
use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    self, parse, parse::Parse, parse_macro_input, Attribute, Ident, ItemFn, PatType, Path, Token,
};

struct MergeAttrs {
    template: ItemFn,
    function: ItemFn,
}

impl Parse for MergeAttrs {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let template = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let function = input.parse()?;
        Ok(Self { template, function })
    }
}

#[cfg(sanitize_multiple_should_panic_compiler_bug)]
fn is_should_panic(attr: &syn::Attribute) -> bool {
    let should_panic: Ident = syn::parse_str("should_panic").unwrap();
    attr.path.is_ident(&should_panic)
}

#[cfg(sanitize_multiple_should_panic_compiler_bug)]
fn sanitize_should_panic_duplication_bug(
    mut attributes: Vec<syn::Attribute>,
) -> Vec<syn::Attribute> {
    if attributes.len() != 2 || attributes[0] != attributes[1] || !is_should_panic(&attributes[0]) {
        // Nothing to do
        return attributes;
    }
    attributes.pop();
    attributes
}

fn collect_template_args(template: &ItemFn) -> HashMap<&Ident, &PatType> {
    template
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(a) => Some(a),
            _ => None,
        })
        .filter_map(|arg| match *arg.pat {
            syn::Pat::Ident(ref id) => Some((&id.ident, arg)),
            _ => None,
        })
        .collect()
}

fn merge_arg_attributes(dest: &mut Vec<Attribute>, source: &[Attribute]) {
    for s in source.iter() {
        if !dest.contains(s) {
            dest.push(s.clone())
        }
    }
}

fn resolve_template_arg<'a>(
    template: &HashMap<&'a Ident, &'a PatType>,
    arg: &Ident,
) -> Option<&'a PatType> {
    let id_name = arg.to_string();
    match (template.get(arg), id_name.starts_with('_')) {
        (Some(&arg), _) => Some(arg),
        (None, true) => template.get(&format_ident!("{}", id_name[1..])).copied(),
        _ => None,
    }
}

fn expand_function_arguments(dest: &mut ItemFn, source: &ItemFn) {
    let to_merge_args = collect_template_args(source);

    for arg in dest.sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(a) = arg {
            if let syn::Pat::Ident(ref id) = *a.pat {
                if let Some(source_arg) = resolve_template_arg(&to_merge_args, &id.ident) {
                    merge_arg_attributes(&mut a.attrs, &source_arg.attrs);
                }
            }
        }
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn merge_attrs(item: TokenStream) -> TokenStream {
    let MergeAttrs {
        template,
        mut function,
    } = parse_macro_input!(item as MergeAttrs);

    expand_function_arguments(&mut function, &template);

    let mut attrs = template.attrs;
    #[cfg(sanitize_multiple_should_panic_compiler_bug)]
    {
        function.attrs = sanitize_should_panic_duplication_bug(function.attrs);
    }
    attrs.append(&mut function.attrs);
    function.attrs = attrs;

    let tokens = quote! {
        #function
    };
    tokens.into()
}

fn get_export(attributes: &[Attribute]) -> Option<&Attribute> {
    attributes
        .iter()
        .find(|&attr| attr.path.is_ident(&format_ident!("export")))
}

/// Define a template where the name is given from the function name. This attribute register all
/// attributes. The function signature don't really mater but to make it clear is better that you
/// use a signature like if you're wrinting a standard `rstest`.
///
/// If you need to export the template at the root of your crate or use it from another crate you
/// should annotate it with `#[export]` attribute. This attribute add `#[macro_export]` attribute to
/// the template macro and make possible to use it from another crate.
///
/// When define a template you can also set the arguments attributes like `#[case]`, `#[values]`
/// and `#[with]`: when you apply it attributes will be copied to the matched by name arguments.
///
#[proc_macro_attribute]
pub fn template(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> TokenStream {
    let mut template: ItemFn = parse(input).unwrap();

    let rstest_index = template
        .attrs
        .iter()
        .position(|attr| attr.path.is_ident(&format_ident!("rstest")));

    let mut attributes = template.attrs;

    template.attrs = match rstest_index {
        Some(idx) => attributes.split_off(idx),
        None => std::mem::take(&mut attributes),
    };

    let mut tokens = match get_export(&attributes) {
        Some(_) => {
            quote! {
                #[macro_export]
            }
        }
        None => quote! {},
    };

    let macro_name = template.sig.ident.clone();
    tokens.extend(quote! {
        /// Apply #macro_name template to given body
        macro_rules! #macro_name {
            ( $test:item ) => {
                        $crate::rstest_reuse::merge_attrs! {
                            #template,
                            $test
                        }
                    }
        }
    });
    tokens.into()
}

/// Apply a defined template. The function signature should satisfy the template attributes
/// but can also add some other fixtures.
/// Example:
///
/// ```
/// use rstest::{rstest, fixture};
/// use rstest_reuse::{self, *};
///
/// #[fixture]
/// fn empty () -> Vec<u32> {
///     Vec::new()    
/// }
///
/// #[template]
/// #[rstest]
/// #[case(2, 2)]
/// #[case(4/2, 2)]
/// fn two_simple_cases(#[case] a: u32, #[case] b: u32) {}
///
/// #[apply(two_simple_cases)]
/// fn it_works(mut empty: Vec<u32>, a: u32, b: u32) {
///     empty.append(a);
///     assert!(empty.last() == b);
/// }
/// ```
/// When use `#[apply]` you can also
/// 1. Ignore an argument by underscore
/// 2. add some cases
/// 3. add some values
///
///
/// ```
/// use rstest::{rstest, fixture};
/// use rstest_reuse::{self, *};
///
/// #[fixture]
/// fn fix (#[default(0)] inner: u32) -> u32 {
///     inner
/// }
///
/// #[template]
/// #[rstest]
/// #[case(2, 2)]
/// #[case(4/2, 2)]
/// fn two_simple_cases(#[case] a: u32, #[case] b: u32) {}
///
/// #[apply(two_simple_cases)]
/// // Add a case
/// #[case(9/3, 3)]
/// // Use fixture with 42 as argument
/// // Ignore b case values
/// // add 2 cases with other in 4, 5 for each case
/// fn lot_of_tests(fix: u32, a: u32, _b: u32, #[values(4, 5)] other: u32) {
///     assert_eq!(fix, 42);
///     assert_eq!(a, 2);
///     assert!([4, 5].contains(other));
/// }
/// ```
///

#[proc_macro_attribute]
pub fn apply(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> TokenStream {
    let template: Path = parse(args).unwrap();
    let test: ItemFn = parse(input).unwrap();
    let tokens = quote! {
        #template! {
            #test
        }
    };
    tokens.into()
}
