use std::collections::HashMap;

use itertools::Itertools;
use ropey::Rope;
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};
use tree_sitter as ts;

use crate::language::{bend, bend_parser, conversion};

use super::document::Document;

lazy_static::lazy_static! {
    pub static ref NAME_TO_TOKEN_TYPE: HashMap<&'static str, SemanticTokenType> = {
        HashMap::from([
            ("variable", SemanticTokenType::VARIABLE),
            ("variable.parameter", SemanticTokenType::PARAMETER),
            ("variable.member", SemanticTokenType::ENUM_MEMBER),
            ("property", SemanticTokenType::PROPERTY),
            ("keyword", SemanticTokenType::KEYWORD),
            ("keyword.conditional", SemanticTokenType::KEYWORD),
            ("keyword.function", SemanticTokenType::KEYWORD),
            ("keyword.return", SemanticTokenType::KEYWORD),
            ("keyword.repeat", SemanticTokenType::KEYWORD),
            ("keyword.type", SemanticTokenType::KEYWORD),
            ("string", SemanticTokenType::STRING),
            ("function", SemanticTokenType::FUNCTION),
            ("function.call", SemanticTokenType::FUNCTION),
            ("type", SemanticTokenType::TYPE),
            // ("constructor", SemanticTokenType::?),
            ("character", SemanticTokenType::STRING),
            ("character.special", SemanticTokenType::STRING),
            ("number", SemanticTokenType::NUMBER),
            ("number.float", SemanticTokenType::NUMBER),
            ("comment", SemanticTokenType::COMMENT),
            // ("punctuation", SemanticTokenType::?),
            // ("punctuation.delimiter", SemanticTokenType::?),
            // ("punctuation.bracket", SemanticTokenType::?),
            ("operator", SemanticTokenType::OPERATOR),
        ])
    };

    pub static ref LEGEND_TOKEN_TYPE: Vec<SemanticTokenType> =
        NAME_TO_TOKEN_TYPE.values().map(|v| v.clone()).unique().collect();

    pub static ref TOKEN_TYPE_INDEX: HashMap<SemanticTokenType, usize> =
        LEGEND_TOKEN_TYPE.iter().enumerate().map(|(i, v)| (v.clone(), i)).collect();
}

// pub const LEGEND_TYPE: &[SemanticTokenType] = &[
//     SemanticTokenType::FUNCTION,
//     SemanticTokenType::VARIABLE,
//     SemanticTokenType::STRING,
//     SemanticTokenType::COMMENT,
//     SemanticTokenType::NUMBER,
//     SemanticTokenType::KEYWORD,
//     SemanticTokenType::OPERATOR,
//     SemanticTokenType::PARAMETER,
// ];

pub fn semantic_tokens(doc: &Document) -> Vec<SemanticToken> {
    let mut cursor = ts::QueryCursor::new();
    let query = ts::Query::new(&bend(), QUERY).unwrap();
    let names = query.capture_names();
    let root = doc.tree.as_ref().unwrap().root_node();

    let mut res = vec![];
    for matche in cursor.matches(&query, root, &TextProviderRope(&doc.text)) {
        for capture in matche.captures {
            let maybe_idx = names
                .get(capture.index as usize)
                .and_then(|name| NAME_TO_TOKEN_TYPE.get(name))
                .and_then(|typ| TOKEN_TYPE_INDEX.get(typ));

            if let Some(&idx) = maybe_idx {
                println!(
                    "Token: {:?}",
                    doc.text.get_byte_slice(capture.node.byte_range())
                );

                res.push(
                    MySemanticToken {
                        range: capture.node.range().into(),
                        token_type: idx,
                    }
                    .into(),
                );
            }
        }
    }

    res
}

#[derive(Debug)]
pub struct MySemanticToken {
    pub range: conversion::Range,
    pub token_type: usize,
}

impl From<MySemanticToken> for SemanticToken {
    fn from(value: MySemanticToken) -> Self {
        SemanticToken {
            delta_line: todo!(),
            delta_start: todo!(),
            length: todo!(),
            token_type: todo!(),
            token_modifiers_bitset: todo!(),
        }
    }
}

#[test]
fn token_capture_test() {
    let code: Rope = "main = (f \"hi!\")".into();
    let mut parser = bend_parser().unwrap();
    let tree = parser.parse(code.to_string(), None).unwrap();

    let query = ts::Query::new(&bend(), &QUERY).unwrap();
    println!("{:?}\n", query.capture_names());

    for qmatch in ts::QueryCursor::new().matches(&query, tree.root_node(), &TextProviderRope(&code))
    {
        for capture in qmatch.captures {
            println!("{:?}", capture);
            let range = capture.node.byte_range();
            println!("{:?}\n", code.slice(range));
        }
    }
}

pub struct TextProviderRope<'a>(pub &'a Rope);

impl<'a> ts::TextProvider<&'a [u8]> for &'a TextProviderRope<'a> {
    type I = ChunksBytes<'a>;
    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        ChunksBytes(self.0.byte_slice(node.byte_range()).chunks())
    }
}

pub struct ChunksBytes<'a>(ropey::iter::Chunks<'a>);

impl<'a> Iterator for ChunksBytes<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|s| s.as_bytes())
    }
}

const QUERY: &'static str = r#"
(identifier) @variable

; TODO: lots of repetitive queries because of this, how can we fix it?
(identifier
  (path) @property
  name: (_) @variable)

(import_name
  "import" @keyword
  (os_path) @string)

(import_from
  "from" @keyword
  (os_path) @string
  "import" @keyword
  (os_path) @string)

(fun_function_definition
  name: (identifier) @function)
(fun_function_definition
  name: (identifier (identifier) @function))

(imp_function_definition
  name: (identifier) @function)
(imp_function_definition
  name: (identifier (identifier) @function))
(parameters) @variable.parameter

(object_definition
  name: (identifier) @type)
(object_definition
  name: (identifier (identifier) @type))
(object_field
  (identifier) @variable.member)
(object_field
  (identifier (identifier) @variable.member))

(imp_type_definition
  name: (identifier) @type)
(imp_type_definition
  name: (identifier (identifier) @type))
(imp_type_constructor
  (identifier) @constructor)
(imp_type_constructor
  (identifier (identifier) @constructor))
(imp_type_constructor_field
  (identifier) @variable.member)
(imp_type_constructor_field
  (identifier (identifier) @variable.member))

(fun_type_definition
  name: (identifier) @type)
(fun_type_definition
  name: (identifier (identifier) @type))
(fun_type_constructor
  (identifier) @constructor)
(fun_type_constructor
  (identifier (identifier) @constructor))
(fun_type_constructor_fields
  (identifier) @variable.member)
(fun_type_constructor_fields
  (identifier (identifier) @variable.member))

(hvm_definition
  name: (identifier) @function)
(hvm_definition
  name: (identifier (identifier) @function))
(hvm_definition
  code: (hvm_code) @string)

(constructor
  (identifier) @constructor)
(constructor
  (identifier (identifier) @constructor))
(constructor
  field: (identifier) @property)

(call_expression
  (identifier) @function.call)
(call_expression
  (identifier (identifier) @function.call))

(switch_case
  (switch_pattern) @character.special
  (#eq? @character.special "_"))

(integer) @number
(float) @number.float

(character) @character
(comment) @comment
[
 (symbol)
 (string)
] @string

[
  ":"
  ","
  ";"
] @punctuation.delimiter

[
  "["
  "]"
  "("
  ")"
  "{"
  "}"
] @punctuation.bracket

[
  "-"
  "-="
  "!="
  "*"
  "**"
  "*="
  "/"
  "/="
  "&"
  "%"
  "^"
  "+"
  "+="
  "<"
  "<="
  "="
  "=="
  ">"
  ">="
  "|"
  "~"
  "&"
  "<-"
  "&="
  "|="
  "^="
  "@="
] @operator

[
  "if"
  "elif"
  "else"
] @keyword.conditional


[
 "def"
  "@"
  "λ"
  "lambda"
  "hvm"
] @keyword.function

"return" @keyword.return
"for" @keyword.repeat

[
  "object"
  "type"
] @keyword.type

[
  "with"
  "match"
  "case"
  "open"
  "use"
  "with"
  "bend"
  "when"
  "fold"
  "switch"
  "ask"
  "let"
  "in"
] @keyword
"#;
