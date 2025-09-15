use tool_tool_base::result::{ToolToolError, ToolToolResult};

#[derive(Debug, Default)]
pub struct TemplateString {
    pub parts: Vec<TemplateStringPart>,
}

impl TemplateString {
    pub fn parts(&self) -> &Vec<TemplateStringPart> {
        &self.parts
    }
}

#[derive(Debug)]
pub enum TemplateStringPart {
    PlainText(String),
    Substitution(TemplateStringSubstitution),
}

impl TemplateStringPart {
    pub fn plain(text: impl Into<String>) -> Self {
        Self::PlainText(text.into())
    }

    pub fn substitution(directive: impl Into<String>) -> Self {
        Self::Substitution(TemplateStringSubstitution {
            directive: directive.into(),
        })
    }
}

#[derive(Debug)]
pub struct TemplateStringSubstitution {
    pub directive: String,
}

impl TemplateString {
    pub fn as_test_string(&self) -> String {
        use std::fmt::Write;
        let mut test_string = String::new();
        for part in &self.parts {
            match part {
                TemplateStringPart::PlainText(text) => {
                    writeln!(test_string, "Plain '{text}'").unwrap();
                }
                TemplateStringPart::Substitution(substitution) => {
                    writeln!(test_string, "Template '{}'", substitution.directive).unwrap();
                }
            }
        }
        test_string
    }
}

impl TryFrom<&str> for TemplateString {
    type Error = ToolToolError;
    fn try_from(value: &str) -> ToolToolResult<Self> {
        let mut parts = vec![];
        let chars: Vec<char> = value.chars().collect();
        let mut start_pos = 0;
        let mut current_pos = 0;
        while current_pos < value.len() {
            if chars[current_pos] == '$'
                && current_pos + 1 < value.len()
                && chars[current_pos + 1] == '{'
            {
                if start_pos < current_pos {
                    parts.push(TemplateStringPart::plain(&value[start_pos..current_pos]));
                }
                current_pos += 2;
                start_pos = current_pos;
                while current_pos < value.len() && chars[current_pos] != '}' {
                    current_pos += 1;
                }
                // TODO: handle missing closing }
                // TODO: handle empty string
                if start_pos < current_pos {
                    parts.push(TemplateStringPart::substitution(
                        &value[start_pos..current_pos],
                    ));
                }
                current_pos += 1;
                start_pos = current_pos;
            } else {
                current_pos += 1;
            }
        }
        if start_pos < current_pos {
            parts.push(TemplateStringPart::plain(&value[start_pos..current_pos]));
        }
        Ok(Self { parts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{Expect, expect};

    #[test]
    fn test_as_test_string() {
        let template_string = TemplateString {
            parts: vec![
                TemplateStringPart::plain("Hello"),
                TemplateStringPart::substitution("foo"),
                TemplateStringPart::substitution("bar"),
                TemplateStringPart::plain("world!"),
            ],
        };
        assert_eq!(
            template_string.as_test_string(),
            "Plain 'Hello'\nTemplate 'foo'\nTemplate 'bar'\nPlain 'world!'\n"
        );
    }

    fn test_parse(template_string: &str, expected: Expect) {
        let parsed = TemplateString::try_from(template_string).unwrap();
        expected.assert_eq(&parsed.as_test_string());
    }

    macro_rules! test_parse {
        ($name: ident, $template_string: expr, $expected: expr) => {
            #[test]
            fn $name() {
                test_parse($template_string, $expected);
            }
        };
    }

    test_parse!(empty, "", expect![""]);

    test_parse!(
        plain,
        "hello world",
        expect![
            r#"
            Plain 'hello world'
        "#
        ]
    );

    test_parse!(
        all_template,
        "${version}",
        expect![[r#"
            Template 'version'
        "#]]
    );

    test_parse!(
        mixed_1,
        "foo${bar}baz${fizz}buzz",
        expect![[r#"
            Plain 'foo'
            Template 'bar'
            Plain 'baz'
            Template 'fizz'
            Plain 'buzz'
        "#]]
    );
    test_parse!(
        mixed_2,
        "${blip}foo${bar}baz${fizz}buzz${blob}${blab}",
        expect![[r#"
            Template 'blip'
            Plain 'foo'
            Template 'bar'
            Plain 'baz'
            Template 'fizz'
            Plain 'buzz'
            Template 'blob'
            Template 'blab'
        "#]]
    );
}
