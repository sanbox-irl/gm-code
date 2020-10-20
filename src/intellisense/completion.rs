use lsp_types::{CompletionItem, CompletionItemKind, MarkupContent};

use crate::services::GmManual;

pub fn completion(input_str: &str, gm_manual: &GmManual) -> Vec<CompletionItem> {
    let mut output = vec![];

    // check for functions:
    for func in gm_manual.functions.values() {
        if func.name.contains(input_str) {
            output.push(CompletionItem {
                label: func.name.clone(),
                kind: Some(CompletionItemKind::Function),
                detail: Some(func.name.clone()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: func.description.clone(),
                })),
                ..CompletionItem::default()
            })
        }
    }

    // check for variables:

    // check for constants:

    output
}
