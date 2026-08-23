//! Tree-sitter based syntax spans for the debugger source view.

use tree_sitter::{Node, Parser};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HighlightKind {
    Text,
    Comment,
    Keyword,
    Mnemonic,
    Label,
    Number,
    String,
    Register,
    Error,
}

#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

pub struct GazmSyntax {
    source: String,
    spans: Vec<Span>,
    line_starts: Vec<usize>,
}

unsafe extern "C" {
    fn tree_sitter_gazm() -> *const tree_sitter::ffi::TSLanguage;
}

impl GazmSyntax {
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter::Language::from_raw(tree_sitter_gazm()) };
        parser
            .set_language(&language)
            .map_err(|error| format!("Gazm Tree-sitter language: {error}"))?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Gazm Tree-sitter parse failed".to_string())?;

        let mut line_starts = vec![0];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(offset + 1);
            }
        }
        let mut spans = Vec::new();
        collect_spans(tree.root_node(), &mut spans);
        spans.sort_by_key(|span| (span.start, span.end));

        Ok(Self {
            source: source.to_owned(),
            spans,
            line_starts,
        })
    }

    pub fn line_spans(&self, line: usize) -> Vec<(String, HighlightKind)> {
        let Some(&line_start) = self.line_starts.get(line) else {
            return Vec::new();
        };
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.source.len())
            .saturating_sub(usize::from(line + 1 < self.line_starts.len()));
        if line_start >= line_end {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut cursor = line_start;
        for span in self
            .spans
            .iter()
            .filter(|span| span.end > line_start && span.start < line_end && span.start < span.end)
        {
            let start = span.start.max(line_start).min(line_end);
            let end = span.end.min(line_end);
            if start > cursor {
                result.push((self.source[cursor..start].to_owned(), HighlightKind::Text));
            }
            if end > start {
                result.push((self.source[start..end].to_owned(), span.kind));
                cursor = end;
            }
        }
        if cursor < line_end {
            result.push((
                self.source[cursor..line_end].to_owned(),
                HighlightKind::Text,
            ));
        }
        merge_adjacent(&mut result);
        result
    }
}

fn collect_spans(node: Node<'_>, spans: &mut Vec<Span>) {
    if let Some(kind) = highlight_kind(node.kind()) {
        spans.push(Span {
            start: node.start_byte(),
            end: node.end_byte(),
            kind,
        });
        // Parent captures such as comments and strings should own their text.
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_spans(child, spans);
    }
}

fn highlight_kind(kind: &str) -> Option<HighlightKind> {
    Some(match kind {
        "comment" | "doc" | "doc_text" => HighlightKind::Comment,
        "string_literal" => HighlightKind::String,
        "dec_num" | "hex_num" | "bin_num" => HighlightKind::Number,
        "mnemonic" => HighlightKind::Mnemonic,
        "label" | "local_label" => HighlightKind::Label,
        "org" | "scope" | "fdb" | "fcb" | "fill" | "rmb" | "fcc" | "include" | "importer"
        | "writebin" | "setdp" | "bsz" | "zmb" | "zmd" | "exec_addr" => HighlightKind::Keyword,
        "a" | "b" | "d" | "x" | "y" | "s" | "u" | "dp" | "pc" => HighlightKind::Register,
        // The immediate node contains a number or symbol; leave it to the
        // child node so `#$2A` receives number colouring instead of one flat
        // operand colour.
        "immediate" => return None,
        "ERROR" => HighlightKind::Error,
        _ => return None,
    })
}

fn merge_adjacent(spans: &mut Vec<(String, HighlightKind)>) {
    let mut merged: Vec<(String, HighlightKind)> = Vec::with_capacity(spans.len());
    for (text, kind) in spans.drain(..) {
        if let Some((previous, previous_kind)) = merged.last_mut() {
            if *previous_kind == kind {
                previous.push_str(&text);
                continue;
            }
        }
        merged.push((text, kind));
    }
    *spans = merged;
}

#[cfg(test)]
mod tests {
    use super::{GazmSyntax, HighlightKind};

    #[test]
    fn highlights_basic_gazm_line() {
        let syntax = GazmSyntax::parse("START: lda #$2A ; hello\n").unwrap();
        let spans = syntax.line_spans(0);
        assert!(spans
            .iter()
            .any(|(text, kind)| { text == "START" && *kind == HighlightKind::Label }));
        assert!(spans
            .iter()
            .any(|(text, kind)| { text == "lda" && *kind == HighlightKind::Mnemonic }));
        assert!(spans
            .iter()
            .any(|(text, kind)| { text == "$2A" && *kind == HighlightKind::Number }));
    }
}
