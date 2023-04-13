use crate::ir::*;

pub fn adapt_rename_all(segment: &mut RustSegment) -> Option<()> {
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum CaseConvention {
        Lower,
        Upper,
        Snake,
        Pascal,
        ScreamingSnake,
    }
    use CaseConvention::*;
    /// expect input follows one of above case
    fn detect_case(s: &str) -> CaseConvention {
        if s.starts_with(char::is_uppercase) {
            if s.contains('_') {
                ScreamingSnake
            } else if s.chars().all(char::is_uppercase) {
                Upper
            } else {
                Pascal
            }
        } else if s.contains('_') {
            Snake
        } else {
            Lower
        }
    }
    impl CaseConvention {
        fn cast(&mut self, other: Self) -> Option<()> {
            if self == &other {
                return Some(());
            }
            match (&self, &other) {
                (Lower, Snake) | (Upper, ScreamingSnake) => {
                    *self = other;
                }
                (Snake, Lower) | (ScreamingSnake, Upper) => {
                    // do nothing
                }
                _ => None?,
            }
            Some(())
        }
        fn into_rename_rule(self) -> RenameRule {
            match self {
                Lower | Snake => RenameRule::SnakeCase,
                Pascal => RenameRule::PascalCase,
                Upper | ScreamingSnake => RenameRule::ScreamingSnakeCase,
            }
        }
    }
    if let RustSegment::Enum(re) = segment {
        let mut conv: Option<CaseConvention> = None;
        for memb in &re.member {
            let s = match memb {
                RustEnumMember::Nullary(v) => v,
                RustEnumMember::Unary(v) => v,
                RustEnumMember::UnaryNamed { variant_name, .. } => variant_name,
            };
            match conv.as_mut() {
                Some(conv) => {
                    let new = detect_case(s);
                    conv.cast(new)?
                }
                None => {
                    conv = Some(detect_case(s));
                }
            };
        }
        let rr = conv?.into_rename_rule();
        if rr.is_pascal_case() {
            return None;
        }
        for memb in &mut re.member {
            match memb {
                RustEnumMember::Unary(v) => {
                    let type_name = v.to_owned();
                    rr.convert_to_pascal(v);
                    *memb = RustEnumMember::UnaryNamed {
                        variant_name: v.to_owned(),
                        type_name,
                    };
                }
                RustEnumMember::Nullary(variant_name)
                | RustEnumMember::UnaryNamed { variant_name, .. } => {
                    rr.convert_to_pascal(variant_name);
                }
            };
        }
        re.attr
            .add_attr(RustStructAttr::Serde(SerdeContainerAttr::RenameAll(rr)));
    }
    Some(())
}