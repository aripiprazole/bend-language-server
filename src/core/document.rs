use std::path::Path;

use bend::diagnostics::{DiagnosticsConfig, Severity};
use bend::fun::load_book::load_to_book;
use ropey::Rope;
use tower_lsp::lsp_types as lsp;
use tree_sitter as ts;
use tree_sitter_highlight::{self as hg, Highlighter};

use crate::language::{bend, bend_parser};
use crate::utils::rope::TextProviderRope;

/// Represents a text document open in the client's text editor.
pub struct Document {
    pub url: lsp::Url,
    pub text: Rope,
    pub tree: Option<ts::Tree>,
    pub parser: ts::Parser,
    pub highlighter: hg::Highlighter,
    pub ast: bend::fun::Book, // pub components: HashMap<String, ComponentInfo>
}

impl Document {
    /// Create an empty document for `url`.
    pub fn new(url: lsp::Url) -> Self {
        Self {
            url,
            text: Rope::new(),
            tree: None,
            parser: bend_parser().unwrap(),
            highlighter: Highlighter::new(),
            ast: bend::fun::Book::default(),
        }
    }

    /// Create a new document with text for `url`.
    pub fn new_with_text(url: lsp::Url, text: &str) -> Self {
        let mut doc = Self::new(url);
        doc.update_whole_text(text);
        doc
    }

    /// Update the document with entirely new text.
    pub fn update_whole_text(&mut self, text: &str) {
        self.text = Rope::from_str(text);
        self.tree = self.parser.parse(text, None);
    }

    pub fn get_tree(&mut self) -> &ts::Tree {
        if self.tree.is_none() {
            self.do_parse();
        }
        self.tree.as_ref().expect("tried to get empty tree")
    }

    /// Find up to one node based on a tree-sitter query.
    pub fn find_one(&self, query: &str) -> Option<ts::Node> {
        let mut cursor = ts::QueryCursor::new();
        let query = ts::Query::new(&bend(), query).unwrap();
        let root = self.tree.as_ref()?.root_node();

        cursor
            .captures(&query, root, &TextProviderRope(&self.text))
            .flat_map(|(m, _)| m.captures)
            .next()
            .map(|capture| capture.node)
    }

    pub fn find_many(&self, query: &str) -> Option<Vec<ts::Node>> {
        let mut cursor = ts::QueryCursor::new();
        let query = ts::Query::new(&bend(), query).unwrap();
        let root = self.tree.as_ref()?.root_node();

        Some(
            cursor
                .captures(&query, root, &TextProviderRope(&self.text))
                .flat_map(|(m, _)| m.captures)
                .map(|capture| capture.node)
                .collect(),
        )
    }

    fn do_parse(&mut self) -> Option<ts::Tree> {
        self.parser.parse_with(
            &mut |start_byte, _| {
                self.text
                    .byte_slice(start_byte..)
                    .chunks()
                    .next()
                    .unwrap_or("")
            },
            self.tree.as_ref(),
        )
    }
}
