use thin_vec::ThinVec;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ScopeSyntaxError {
    #[error("scope separator cannot be empty")]
    EmptySeparator,
    #[error("qualified name cannot be empty")]
    EmptyName,
    #[error("qualified name contains an empty component")]
    EmptyComponent,
}

/// Syntax used to parse and format qualified scope names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeSyntax {
    separator: String,
}

impl ScopeSyntax {
    /// Construct syntax using one token for nesting and absolute paths.
    pub fn new(separator: impl Into<String>) -> Self {
        Self {
            separator: separator.into(),
        }
    }

    pub fn try_new(separator: impl Into<String>) -> Result<Self, ScopeSyntaxError> {
        let syntax = Self::new(separator);
        if syntax.separator.is_empty() {
            Err(ScopeSyntaxError::EmptySeparator)
        } else {
            Ok(syntax)
        }
    }

    pub fn separator(&self) -> &str {
        &self.separator
    }
}

impl Default for ScopeSyntax {
    fn default() -> Self {
        Self::new("::")
    }
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct ScopedName<'a> {
    input: &'a str,
    symbol: &'a str,
    pub path: ThinVec<&'a str>,
    absolute: bool,
    syntax: ScopeSyntax,
}

impl<'a> std::fmt::Display for ScopedName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}
// @TODO compose scope name with scope path and remove dupe code
impl<'a> ScopedName<'a> {
    pub fn is_abs(&self) -> bool {
        self.absolute
    }

    pub fn is_relative(&self) -> bool {
        !self.is_abs()
    }

    pub fn symbol(&self) -> &str {
        self.symbol
    }

    pub fn path(&self) -> &[&str] {
        &self.path
    }

    pub fn path_as_string(&self) -> String {
        self.path_as_string_with(&self.syntax)
    }

    pub fn path_as_string_with(&self, syntax: &ScopeSyntax) -> String {
        let path = self.path.join(syntax.separator());
        if self.is_abs() {
            format!("{}{path}", syntax.separator())
        } else {
            path
        }
    }

    pub fn as_relative(mut self) -> Self {
        if self.is_abs() {
            self.absolute = false;
            self
        } else {
            self
        }
    }

    pub fn new(input: &'a str) -> Self {
        Self::parse(input, &ScopeSyntax::default())
    }

    pub fn parse(input: &'a str, syntax: &ScopeSyntax) -> Self {
        // Keep the historical infallible API, but never panic on malformed input.
        // Call `try_parse` when invalid source should be diagnosed.
        Self::parse_parts(input, syntax)
    }

    pub fn try_parse(input: &'a str, syntax: &ScopeSyntax) -> Result<Self, ScopeSyntaxError> {
        if input.is_empty() {
            return Err(ScopeSyntaxError::EmptyName);
        }
        if syntax.separator.is_empty() {
            return Err(ScopeSyntaxError::EmptySeparator);
        }
        let parsed = Self::parse_parts(input, syntax);
        if parsed.symbol.is_empty() || parsed.path.iter().any(|part| part.is_empty()) {
            return Err(ScopeSyntaxError::EmptyComponent);
        }
        Ok(parsed)
    }

    fn parse_parts(input: &'a str, syntax: &ScopeSyntax) -> Self {
        let absolute = input.starts_with(syntax.separator());
        let body = if absolute {
            &input[syntax.separator().len()..]
        } else {
            input
        };
        let splits: ThinVec<_> = body.split(syntax.separator()).collect();
        let len = splits.len();

        let (path, symbol) = (
            &splits[..len.saturating_sub(1)],
            splits.last().copied().unwrap_or(""),
        );

        let path: ThinVec<_> = path.iter().copied().collect();

        Self {
            input,
            symbol,
            path,
            absolute,
            syntax: syntax.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScopePath<'a> {
    input: &'a str,
    pub path_parts: ThinVec<&'a str>,
    absolute: bool,
    syntax: ScopeSyntax,
}

impl<'a> std::fmt::Display for ScopePath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}

impl<'a> ScopePath<'a> {
    pub fn is_abs(&self) -> bool {
        self.absolute
    }

    pub fn is_relative(&self) -> bool {
        !self.is_abs()
    }

    pub fn path(&self) -> &[&str] {
        &self.path_parts
    }

    pub fn path_as_string(&self) -> String {
        self.path_as_string_with(&self.syntax)
    }

    pub fn path_as_string_with(&self, syntax: &ScopeSyntax) -> String {
        let path = self.path_parts.join(syntax.separator());
        if self.is_abs() {
            format!("{}{path}", syntax.separator())
        } else {
            path
        }
    }

    pub fn as_relative(mut self) -> Self {
        if self.is_abs() {
            self.absolute = false;
            self
        } else {
            self
        }
    }

    pub fn new(input: &'a str) -> Self {
        Self::parse(input, &ScopeSyntax::default())
    }

    pub fn parse(input: &'a str, syntax: &ScopeSyntax) -> Self {
        Self::parse_parts(input, syntax)
    }

    pub fn try_parse(input: &'a str, syntax: &ScopeSyntax) -> Result<Self, ScopeSyntaxError> {
        if input.is_empty() {
            return Err(ScopeSyntaxError::EmptyName);
        }
        if syntax.separator.is_empty() {
            return Err(ScopeSyntaxError::EmptySeparator);
        }
        let parsed = Self::parse_parts(input, syntax);
        if parsed.path_parts.iter().any(|part| part.is_empty()) {
            return Err(ScopeSyntaxError::EmptyComponent);
        }
        Ok(parsed)
    }

    fn parse_parts(input: &'a str, syntax: &ScopeSyntax) -> Self {
        let absolute = input.starts_with(syntax.separator());
        let body = if absolute {
            &input[syntax.separator().len()..]
        } else {
            input
        };
        let path: ThinVec<_> = body.split(syntax.separator()).collect();

        Self {
            input,
            path_parts: path,
            absolute,
            syntax: syntax.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_names_are_reported_without_panicking() {
        let syntax = ScopeSyntax::default();
        assert!(matches!(
            ScopedName::try_parse("", &syntax),
            Err(ScopeSyntaxError::EmptyName)
        ));
        assert!(matches!(
            ScopedName::try_parse("::a::::b", &syntax),
            Err(ScopeSyntaxError::EmptyComponent)
        ));
        assert!(matches!(
            ScopePath::try_parse("", &syntax),
            Err(ScopeSyntaxError::EmptyName)
        ));
        assert_eq!(
            ScopeSyntax::try_new(""),
            Err(ScopeSyntaxError::EmptySeparator)
        );
    }

    #[test]
    fn one_separator_configures_both_forms() {
        let syntax = ScopeSyntax::new("::");
        assert_eq!(syntax.separator(), "::");
        assert_eq!(syntax.separator(), "::");
    }
}
