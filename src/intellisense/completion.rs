use super::utils;
use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, MarkedString, MarkupContent};

use crate::services::GmManual;

use super::utils::StdCompletionKind;

pub fn initial_completion(input_str: &str, gm_manual: &GmManual) -> CompletionList {
    let mut output = vec![];

    // check for functions:
    for func in gm_manual.functions.values() {
        if func.name.contains(input_str) {
            output.push(CompletionItem {
                label: func.name.clone(),
                kind: Some(CompletionItemKind::Function),
                data: serde_json::to_value(StdCompletionKind::Function).ok(),
                ..CompletionItem::default()
            })
        }
    }

    // check for variables:
    for variable in gm_manual.variables.values() {
        if variable.name.contains(input_str) {
            output.push(CompletionItem {
                label: variable.name.clone(),
                kind: Some(CompletionItemKind::Variable),
                data: serde_json::to_value(StdCompletionKind::Variable).ok(),

                ..CompletionItem::default()
            })
        }
    }

    // check for constants:
    for constant in gm_manual.constants.values() {
        if constant.name.contains(input_str) {
            output.push(CompletionItem {
                label: constant.name.clone(),
                kind: Some(CompletionItemKind::Value),
                data: serde_json::to_value(StdCompletionKind::Constant).ok(),

                ..CompletionItem::default()
            })
        }
    }

    CompletionList {
        is_incomplete: true,
        items: output,
    }
}

pub fn resolve_completion(mut completion: CompletionItem, gm_manual: &GmManual) -> CompletionItem {
    if let Some(data) = completion.data.clone() {
        if let Ok(v) = serde_json::from_value(data) {
            if let Some(output) = utils::detailed_docs_data(&completion.label, &[v], gm_manual) {
                completion.detail = Some(output.detail);
                let documentation = output
                    .description
                    .into_iter()
                    .map(|v| match v {
                        MarkedString::String(v) => v,
                        MarkedString::LanguageString(l) => format!("```\n{}\n```", l.value),
                    })
                    .join("\n");

                completion.documentation =
                    Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value: documentation,
                    }));
            }
        }
    }

    completion
}
