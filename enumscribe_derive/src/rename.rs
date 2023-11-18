use proc_macro2::Span;

use crate::error::{MacroResult, MacroError};

#[derive(Clone, Copy, Debug)]
pub(crate) enum RenameVariant {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

impl RenameVariant {
    pub(crate) fn from_str(s: &str, span: Span) -> MacroResult<Self> {
        // Shame we can't use enumscribe for this...
        match s {
            "lowercase" => Ok(Self::Lower),
            "UPPERCASE" => Ok(Self::Upper),
            "PascalCase" => Ok(Self::Pascal),
            "camelCase" => Ok(Self::Camel),
            "snake_case" => Ok(Self::Snake),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnake),
            "kebab-case" => Ok(Self::Kebab),
            "SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebab),
            _ => Err(MacroError::new(
                format!(
                    "invalid case {:?} (allowed values are: \
                     lowercase, \
                     UPPERCASE, \
                     PascalCase, \
                     camelCase, \
                     snake_case, \
                     SCREAMING_SNAKE_CASE, \
                     kebab-case, \
                     SCREAMING-KEBAB-CASE)",
                    s
                ),
                span
            )),
        }
    }
    
    pub(crate) fn apply(self, s: &str) -> String {
        match self {
            RenameVariant::Lower => s.to_lowercase(),
            RenameVariant::Upper => s.to_uppercase(),
            RenameVariant::Pascal => PascalCase.convert_enum_variant(s),
            RenameVariant::Camel => CamelCase.convert_enum_variant(s),
            RenameVariant::Snake => SnakeCase(CharCase::Lower).convert_enum_variant(s),
            RenameVariant::ScreamingSnake => SnakeCase(CharCase::Upper).convert_enum_variant(s),
            RenameVariant::Kebab => KebabCase(CharCase::Lower).convert_enum_variant(s),
            RenameVariant::ScreamingKebab => KebabCase(CharCase::Upper).convert_enum_variant(s),
        }
    }
}

trait WordAwareCase {
    fn convert_enum_variant(&self, s: &str) -> String {
        let mut converted = String::new();
        let mut component = String::new();
        let mut prev_case = Option::None;

        for c in s.chars() {
            let case = CharCase::of(c);

            let (push_component, push_char) = {
                if matches!((prev_case, case), (Some(CharCase::Lower), Some(CharCase::Upper))) {
                    (true, true)
                } else if c == '_' {
                    (true, false)
                } else {
                    (false, true)
                }
            };

            if push_component && !component.is_empty() {
                self.push_word(&mut converted, &component);
                component.clear();
            }

            if push_char {
                component.push(c);
            }
            
            prev_case = case;
        }

        if !component.is_empty() {
            self.push_word(&mut converted, &component);
        }

        converted
    }
    
    fn push_word(&self, buf: &mut String, word: &str);
}

struct PascalCase;

impl WordAwareCase for PascalCase {
    fn push_word(&self, buf: &mut String, word: &str) {
        if let Some((head, tail)) = str_head_tail(word) {
            buf.extend(head.to_uppercase());
            buf.push_str(&tail.to_lowercase());
        }
    }
}

struct CamelCase;

impl WordAwareCase for CamelCase {
    fn push_word(&self, buf: &mut String, word: &str) {
        if buf.is_empty() {
            buf.push_str(&word.to_lowercase());
        } else if let Some((head, tail)) = str_head_tail(word) {
            buf.extend(head.to_uppercase());
            buf.push_str(&tail.to_lowercase());
        }
    }
}

struct SnakeCase(CharCase);

impl WordAwareCase for SnakeCase {
    fn push_word(&self, buf: &mut String, word: &str) {
        if !buf.is_empty() {
            buf.push('_');
        }
        buf.push_str(&self.0.convert(word));
    }
}

struct KebabCase(CharCase);

impl WordAwareCase for KebabCase {
    fn push_word(&self, buf: &mut String, word: &str) {
        if !buf.is_empty() {
            buf.push('-');
        }
        buf.push_str(&self.0.convert(word));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CharCase {
    Upper,
    Lower,
}

impl CharCase {
    fn of(c: char) -> Option<Self> {
        if c.is_uppercase() {
            Some(Self::Upper)
        } else if c.is_lowercase() {
            Some(Self::Lower)
        } else {
            None
        }
    }

    fn convert(self, s: &str) -> String {
        match self {
            Self::Upper => s.to_uppercase(),
            Self::Lower => s.to_lowercase(),
        }
    }
}

fn str_head_tail(s: &str) -> Option<(char, &str)> {
    let head = s.chars().next()?;
    let tail = &s[head.len_utf8()..];
    Some((head, tail))
}

#[cfg(test)]
mod test {
    use super::{PascalCase, CamelCase, SnakeCase, KebabCase, CharCase, WordAwareCase};
    
    #[test]
    fn test_pascal_case() {
        assert_eq!(PascalCase.convert_enum_variant(""), "");
        assert_eq!(PascalCase.convert_enum_variant("foo"), "Foo");
        assert_eq!(PascalCase.convert_enum_variant("fooBaa"), "FooBaa");
        assert_eq!(PascalCase.convert_enum_variant("FooBaa"), "FooBaa");
        assert_eq!(PascalCase.convert_enum_variant("foo_baa"), "FooBaa");
        assert_eq!(PascalCase.convert_enum_variant("FOO_BAA"), "FooBaa");
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(CamelCase.convert_enum_variant(""), "");
        assert_eq!(CamelCase.convert_enum_variant("foo"), "foo");
        assert_eq!(CamelCase.convert_enum_variant("fooBaa"), "fooBaa");
        assert_eq!(CamelCase.convert_enum_variant("FooBaa"), "fooBaa");
        assert_eq!(CamelCase.convert_enum_variant("foo_baa"), "fooBaa");
        assert_eq!(CamelCase.convert_enum_variant("FOO_BAA"), "fooBaa");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(SnakeCase(CharCase::Lower).convert_enum_variant(""), "");
        assert_eq!(SnakeCase(CharCase::Lower).convert_enum_variant("foo"), "foo");
        assert_eq!(SnakeCase(CharCase::Lower).convert_enum_variant("fooBaa"), "foo_baa");
        assert_eq!(SnakeCase(CharCase::Lower).convert_enum_variant("FooBaa"), "foo_baa");
        assert_eq!(SnakeCase(CharCase::Lower).convert_enum_variant("foo_baa"), "foo_baa");
        assert_eq!(SnakeCase(CharCase::Lower).convert_enum_variant("FOO_BAA"), "foo_baa");

        assert_eq!(SnakeCase(CharCase::Upper).convert_enum_variant(""), "");
        assert_eq!(SnakeCase(CharCase::Upper).convert_enum_variant("foo"), "FOO");
        assert_eq!(SnakeCase(CharCase::Upper).convert_enum_variant("fooBaa"), "FOO_BAA");
        assert_eq!(SnakeCase(CharCase::Upper).convert_enum_variant("FooBaa"), "FOO_BAA");
        assert_eq!(SnakeCase(CharCase::Upper).convert_enum_variant("foo_baa"), "FOO_BAA");
        assert_eq!(SnakeCase(CharCase::Upper).convert_enum_variant("FOO_BAA"), "FOO_BAA");
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(KebabCase(CharCase::Lower).convert_enum_variant(""), "");
        assert_eq!(KebabCase(CharCase::Lower).convert_enum_variant("foo"), "foo");
        assert_eq!(KebabCase(CharCase::Lower).convert_enum_variant("fooBaa"), "foo-baa");
        assert_eq!(KebabCase(CharCase::Lower).convert_enum_variant("FooBaa"), "foo-baa");
        assert_eq!(KebabCase(CharCase::Lower).convert_enum_variant("foo_baa"), "foo-baa");
        assert_eq!(KebabCase(CharCase::Lower).convert_enum_variant("FOO_BAA"), "foo-baa");

        assert_eq!(KebabCase(CharCase::Upper).convert_enum_variant(""), "");
        assert_eq!(KebabCase(CharCase::Upper).convert_enum_variant("foo"), "FOO");
        assert_eq!(KebabCase(CharCase::Upper).convert_enum_variant("fooBaa"), "FOO-BAA");
        assert_eq!(KebabCase(CharCase::Upper).convert_enum_variant("FooBaa"), "FOO-BAA");
        assert_eq!(KebabCase(CharCase::Upper).convert_enum_variant("foo_baa"), "FOO-BAA");
        assert_eq!(KebabCase(CharCase::Upper).convert_enum_variant("FOO_BAA"), "FOO-BAA");
    }
}
