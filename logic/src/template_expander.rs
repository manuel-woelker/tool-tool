use crate::template_string::{TemplateString, TemplateStringPart, TemplateStringSubstitution};
use std::collections::HashMap;
use tool_tool_base::result::{ToolToolResult, bail};

#[derive(Default)]
pub struct TemplateExpander<'a> {
    pub replacer: HashMap<String, Box<dyn SubstitutionReplacer + 'a>>,
}

pub trait SubstitutionReplacer {
    fn replace(&self, substitution: &TemplateStringSubstitution) -> String;
}

impl<F> SubstitutionReplacer for F
where
    for<'a> F: Fn(&'a TemplateStringSubstitution) -> String,
{
    fn replace(&self, substitution: &TemplateStringSubstitution) -> String {
        self(substitution)
    }
}

impl<'a> TemplateExpander<'a> {
    pub fn add_replacer(
        &mut self,
        key: impl Into<String>,
        replacer: impl SubstitutionReplacer + 'a,
    ) {
        self.replacer.insert(key.into(), Box::new(replacer));
    }
    pub fn add_replace_fn(
        &mut self,
        key: impl Into<String>,
        replacer: impl Fn(&TemplateStringSubstitution) -> String + 'a,
    ) {
        self.replacer.insert(key.into(), Box::new(replacer));
    }

    pub fn expand(&self, template: TemplateString) -> ToolToolResult<String> {
        let mut result = String::new();
        for part in template.parts() {
            match part {
                TemplateStringPart::PlainText(text) => {
                    result.push_str(text);
                }
                TemplateStringPart::Substitution(substitution) => {
                    if let Some(replacer) = self.replacer.get(&substitution.directive) {
                        result.push_str(&replacer.replace(substitution));
                    } else {
                        bail!(
                            "Unknown substitution directive '{}'",
                            substitution.directive
                        );
                    }
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template_string::TemplateString;

    #[test]
    fn test_template_expander() {
        let version = "1.0.0".to_string();
        let borrowed_version = &version;
        let mut expander = TemplateExpander::default();
        expander.add_replace_fn("version", |_| borrowed_version.to_string());
        let actual = expander
            .expand(TemplateString::try_from("foo${version}bar").unwrap())
            .unwrap();
        assert_eq!(actual, "foo1.0.0bar");
    }

    #[test]
    fn test_template_expander_with_arguments() {
        let mut expander = TemplateExpander::default();
        expander.add_replace_fn("fizz", |substitution| substitution.arguments[0].clone());
        let actual = expander
            .expand(TemplateString::try_from("foo${fizz:buzz}bar").unwrap())
            .unwrap();
        assert_eq!(actual, "foobuzzbar");
    }
}
