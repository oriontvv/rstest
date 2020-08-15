/// `fixture`'s related data and parsing
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_quote,
    visit_mut::VisitMut,
    Expr, FnArg, Ident, ItemFn, Token,
};

use super::{parse_vector_trailing_till_double_comma, Attributes, Fixture, Positional};
use crate::parse::Attribute;
use crate::{
    error::ErrorsVec,
    refident::{MaybeIdent, RefIdent},
    utils::attr_is,
};
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(PartialEq, Debug, Default)]
pub(crate) struct FixtureInfo {
    pub(crate) data: FixtureData,
    pub(crate) attributes: FixtureModifiers,
}

impl Parse for FixtureModifiers {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(input.parse::<Attributes>()?.into())
    }
}

impl Parse for FixtureInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.is_empty() {
            Default::default()
        } else {
            Self {
                data: input.parse()?,
                attributes: input
                    .parse::<Token![::]>()
                    .or_else(|_| Ok(Default::default()))
                    .and_then(|_| input.parse())?,
            }
        })
    }
}

impl FixtureInfo {
    pub(crate) fn extend(&mut self, item_fn: &mut ItemFn) -> std::result::Result<(), ErrorsVec> {
        let mut extractor = FixtureFunctionExtractor(self, Vec::new());
        extractor.visit_item_fn_mut(item_fn);
        if extractor.1.len() > 0 {
            Err(extractor.1.into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
struct DefValue(Expr);

impl Parse for DefValue {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![ = ]>()?;
        Ok(DefValue(input.parse::<Expr>()?))
    }
}

/// Simple struct used to visit function attributes and extract fixture info and
/// eventualy parsing errors
struct FixtureFunctionExtractor<'a>(&'a mut FixtureInfo, Vec<syn::Error>);

impl<'a> VisitMut for FixtureFunctionExtractor<'a> {
    fn visit_fn_arg_mut(&mut self, node: &mut FnArg) {
        let name = node.maybe_ident().cloned();
        if name.is_none() {
            return;
        }

        let name = name.unwrap();
        if let FnArg::Typed(ref mut arg) = node {
            // Extract and interesting attributes
            let attrs = std::mem::take(&mut arg.attrs);
            let (attrs, with_atributes): (Vec<_>, Vec<_>) =
                attrs.into_iter().partition(|attr| !attr_is(attr, "with"));
            let (attrs, default_atributes): (Vec<_>, Vec<_>) = attrs
                .into_iter()
                .partition(|attr| !attr_is(attr, "default"));

            arg.attrs = attrs;

            // Parse positionals
            let (positionals, positional_errors): (Vec<_>, Vec<_>) = with_atributes
                .into_iter()
                .map(|attribute| attribute.parse_args::<Positional>())
                .partition(Result::is_ok);

            // Parse Default values
            let (defaults, defult_errors): (Vec<_>, Vec<_>) = default_atributes
                .into_iter()
                .map(|attribute| syn::parse2::<DefValue>(attribute.tokens))
                .partition(Result::is_ok);

            // Collect Infos
            self.0.data.items.extend(
                positionals
                    .into_iter()
                    .map(|p| p.unwrap())
                    .map(|p| Fixture::new(name.clone(), p).into()),
            );
            self.0.data.items.extend(
                defaults
                    .into_iter()
                    .map(|d| d.unwrap())
                    .map(|d| ArgumentValue::new(name.clone(), d.0).into()),
            );
            // Collect Errors
            self.1
                .extend(positional_errors.into_iter().map(|e| e.unwrap_err()));
            self.1
                .extend(defult_errors.into_iter().map(|e| e.unwrap_err()))
        }
    }
}

#[derive(PartialEq, Debug, Default)]
pub(crate) struct FixtureData {
    pub items: Vec<FixtureItem>,
}

impl FixtureData {
    pub(crate) fn fixtures(&self) -> impl Iterator<Item = &Fixture> {
        self.items.iter().filter_map(|f| match f {
            FixtureItem::Fixture(ref fixture) => Some(fixture),
            _ => None,
        })
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &ArgumentValue> {
        self.items.iter().filter_map(|f| match f {
            FixtureItem::ArgumentValue(ref value) => Some(value),
            _ => None,
        })
    }
}

impl Parse for FixtureData {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![::]) {
            Ok(Default::default())
        } else {
            Ok(Self {
                items: parse_vector_trailing_till_double_comma::<_, Token![,]>(input)?,
            })
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct ArgumentValue {
    pub name: Ident,
    pub expr: Expr,
}

impl ArgumentValue {
    pub(crate) fn new(name: Ident, expr: Expr) -> Self {
        Self { name, expr }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum FixtureItem {
    Fixture(Fixture),
    ArgumentValue(ArgumentValue),
}

impl From<Fixture> for FixtureItem {
    fn from(f: Fixture) -> Self {
        FixtureItem::Fixture(f)
    }
}

impl Parse for FixtureItem {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek2(Token![=]) {
            input.parse::<ArgumentValue>().map(|v| v.into())
        } else {
            input.parse::<Fixture>().map(|v| v.into())
        }
    }
}

impl RefIdent for FixtureItem {
    fn ident(&self) -> &Ident {
        match self {
            FixtureItem::Fixture(Fixture { ref name, .. }) => name,
            FixtureItem::ArgumentValue(ArgumentValue { ref name, .. }) => name,
        }
    }
}

impl ToTokens for FixtureItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident().to_tokens(tokens)
    }
}

impl From<ArgumentValue> for FixtureItem {
    fn from(av: ArgumentValue) -> Self {
        FixtureItem::ArgumentValue(av)
    }
}

impl Parse for ArgumentValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let expr = input.parse()?;
        Ok(ArgumentValue::new(name, expr))
    }
}

wrap_attributes!(FixtureModifiers);

impl FixtureModifiers {
    const DEFAULT_RET_ATTR: &'static str = "default";
    const PARTIAL_RET_ATTR: &'static str = "partial_";

    pub(crate) fn extract_default_type(&self) -> Option<syn::ReturnType> {
        self.extract_type(Self::DEFAULT_RET_ATTR)
    }

    pub(crate) fn extract_partial_type(&self, pos: usize) -> Option<syn::ReturnType> {
        self.extract_type(&format!("{}{}", Self::PARTIAL_RET_ATTR, pos))
    }

    fn extract_type(&self, attr_name: &str) -> Option<syn::ReturnType> {
        self.iter()
            .filter_map(|m| match m {
                Attribute::Type(name, t) if name == attr_name => Some(parse_quote! { -> #t}),
                _ => None,
            })
            .next()
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::test::{assert_eq, *};

    mod parse {
        use super::{assert_eq, *};
        use mytest::rstest;

        fn parse_fixture<S: AsRef<str>>(fixture_data: S) -> FixtureInfo {
            parse_meta(fixture_data)
        }

        #[test]
        fn happy_path() {
            let data = parse_fixture(
                r#"my_fixture(42, "other"), other(vec![42]), value=42, other_value=vec![1.0]
                    :: trace :: no_trace(some)"#,
            );

            let expected = FixtureInfo {
                data: vec![
                    fixture("my_fixture", vec!["42", r#""other""#]).into(),
                    fixture("other", vec!["vec![42]"]).into(),
                    arg_value("value", "42").into(),
                    arg_value("other_value", "vec![1.0]").into(),
                ]
                .into(),
                attributes: Attributes {
                    attributes: vec![
                        Attribute::attr("trace"),
                        Attribute::tagged("no_trace", vec!["some"]),
                    ],
                }
                .into(),
            };

            assert_eq!(expected, data);
        }

        #[test]
        fn some_literals() {
            let args_expressions = literal_expressions_str();
            let fixture = parse_fixture(&format!("my_fixture({})", args_expressions.join(", ")));
            let args = fixture.data.fixtures().next().unwrap().positional.clone();

            assert_eq!(to_args!(args_expressions), args.0);
        }

        #[test]
        fn empty_fixtures() {
            let data = parse_fixture(r#"::trace::no_trace(some)"#);

            let expected = FixtureInfo {
                attributes: Attributes {
                    attributes: vec![
                        Attribute::attr("trace"),
                        Attribute::tagged("no_trace", vec!["some"]),
                    ],
                }
                .into(),
                ..Default::default()
            };

            assert_eq!(expected, data);
        }

        #[test]
        fn empty_attributes() {
            let data = parse_fixture(r#"my_fixture(42, "other")"#);

            let expected = FixtureInfo {
                data: vec![fixture("my_fixture", vec!["42", r#""other""#]).into()].into(),
                ..Default::default()
            };

            assert_eq!(expected, data);
        }

        #[rstest(
            input,
            expected,
            case("first(42),", 1),
            case("first(42), second=42,", 2),
            case(r#"fixture(42, "other"), :: trace"#, 1),
            case(r#"second=42, fixture(42, "other"), :: trace"#, 2)
        )]
        fn should_accept_trailing_comma(input: &str, expected: usize) {
            let info: FixtureInfo = input.ast();

            assert_eq!(
                expected,
                info.data.fixtures().count() + info.data.values().count()
            );
        }
    }
}

#[cfg(test)]
mod extend {
    use super::*;
    use crate::test::{assert_eq, *};
    use syn::ItemFn;

    mod should {
        use super::{assert_eq, *};

        #[test]
        fn use_with_attributes() {
            let to_parse = r#"
                fn my_fix(#[with(2)] f1: &str, #[with(vec![1,2], "s")] f2: u32) {}
            "#;

            let mut item_fn: ItemFn = to_parse.ast();
            let mut info = FixtureInfo::default();

            info.extend(&mut item_fn).unwrap();

            let expected = FixtureInfo {
                data: vec![
                    fixture("f1", vec!["2"]).into(),
                    fixture("f2", vec!["vec![1,2]", r#""s""#]).into(),
                ]
                .into(),
                ..Default::default()
            };

            assert!(!format!("{:?}", item_fn).contains("with"));
            assert_eq!(expected, info);
        }

        #[test]
        fn use_default_attributes() {
            let to_parse = r#"
                fn my_fix(#[default = 2] f1: &str, #[default = (vec![1,2], "s")] f2: (Vec<u32>, &str)) {}
            "#;

            let mut item_fn: ItemFn = to_parse.ast();
            let mut info = FixtureInfo::default();

            info.extend(&mut item_fn).unwrap();

            let expected = FixtureInfo {
                data: vec![
                    arg_value("f1", "2").into(),
                    arg_value("f2", r#"(vec![1,2], "s")"#).into(),
                ]
                .into(),
                ..Default::default()
            };

            assert!(!format!("{:?}", item_fn).contains("default"));
            assert_eq!(expected, info);
        }

        #[test]
        fn raise_error_for_invalid_expressions() {
            let mut item_fn: ItemFn = r#"
                fn my_fix(#[with(valid)] f1: &str, #[with(with(,.,))] f2: u32, #[with(with(use))] f3: u32) {}
            "#
            .ast();

            let errors = FixtureInfo::default().extend(&mut item_fn).unwrap_err();

            assert_eq!(2, errors.len());
        }
    }
}
