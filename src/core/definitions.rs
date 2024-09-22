use super::{diagnostics::span_to_range, document::Document};
use bend::fun::Term;
use itertools::Itertools;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    self as lsp, CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
    GotoDefinitionParams, SemanticTokensRangeResult,
};

use crate::server::{self, Backend};

pub async fn goto_definition(
    backend: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<lsp::GotoDefinitionResponse>> {
    Ok(None)
}

pub async fn completion(
    backend: &Backend,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let url = params.text_document_position.text_document.uri;
    let items = backend
        .read_document(&url, |doc| {
            let ident_offset = doc
                .text
                .line(params.text_document_position.position.line as usize)
                .byte_to_char(params.text_document_position.position.character as usize);

            Some(
                find_book_definitions(doc)
                    .into_iter()
                    //.merge(find_local_variables(doc))
                    .filter(|def| {
                        if def.kind == CompletionItemKind::VARIABLE {
                            let start_idx = doc
                                .text
                                .line(def.range.start.line as usize)
                                .byte_to_char(def.range.start.character as usize);
                            let end_idx = doc
                                .text
                                .line(def.range.end.line as usize)
                                .byte_to_char(def.range.end.character as usize);
                            start_idx < ident_offset && end_idx > ident_offset
                        } else {
                            true
                        }
                    })
                    .map(|definition| CompletionItem {
                        label: definition.name,
                        kind: definition.kind.into(),
                        ..Default::default()
                    })
                    .collect_vec(),
            )
        })
        .unwrap_or_default();

    Ok(Some(CompletionResponse::Array(items)))
}

pub fn get_definition_of_expr(expr: Term, params: GotoDefinitionParams) {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    pub kind: CompletionItemKind,
    pub name: String,
    pub range: lsp::Range,
}

/// Diagnostics with `Rule`, `Function` or `Inet` origins may have their
/// spans including entire definitions, while we would only like to
/// highlight their names.
pub fn find_local_variables(doc: &Document) -> Vec<Definition> {
    let bytes = doc.text.bytes().collect_vec();
    doc.find_many(include_str!("find_defs.scm"))
        .unwrap_or_default()
        .into_iter()
        .map(|node| Definition {
            kind: CompletionItemKind::VARIABLE,
            name: node.utf8_text(&bytes).unwrap_or("").to_string(),
            range: super::diagnostics::ts_range_to_lsp(node.range()),
        })
        .collect_vec()
}

pub fn find_book_definitions(doc: &Document) -> Vec<Definition> {
    let mut definitions = vec![];
    for hvm_def in doc.ast.hvm_defs.values() {
        definitions.push(Definition {
            kind: CompletionItemKind::FUNCTION,
            name: hvm_def.name.to_string(),
            range: span_to_range(&hvm_def.source.span),
        })
    }
    for definition in doc.ast.defs.values() {
        definitions.push(Definition {
            kind: CompletionItemKind::FUNCTION,
            name: definition.name.to_string(),
            range: span_to_range(&definition.source.span),
        })
    }
    for (name, adt) in &doc.ast.adts {
        definitions.push(Definition {
            kind: CompletionItemKind::CLASS,
            name: name.to_string(),
            range: span_to_range(&adt.source.span),
        });

        for (name, fields) in &adt.ctrs {
            definitions.push(Definition {
                kind: CompletionItemKind::CONSTRUCTOR,
                name: name.to_string(),
                range: span_to_range(&adt.source.span),
            });

            for field in fields {
                definitions.push(Definition {
                    kind: CompletionItemKind::FIELD,
                    name: field.nam.to_string(),
                    range: span_to_range(&adt.source.span),
                })
            }
        }
    }
    definitions
}

#[cfg(test)]
mod tests {
    use std::path::{absolute, Path};

    use tower_lsp::lsp_types::Url;

    use crate::core::{definitions::find_book_definitions, diagnostics, document::Document};

    #[test]
    fn test_find_definitions() {
        let path = absolute(Path::new("src/core/example.bend")).unwrap();
        let path = Url::from_file_path(path).unwrap();
        let text = include_str!("example.bend");
        let mut document = Document::new_with_text(path, text);
        let _ = diagnostics::check(&mut document);

        let matches = find_book_definitions(&document);

        println!("{:#?}", matches);
    }
}
