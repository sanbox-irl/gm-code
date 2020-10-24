use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, MarkupContent};

use crate::services::GmManual;

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
    if let Some(data) = &completion.data {
        if let Ok(v) = serde_json::from_value(data.clone()) {
            match v {
                StdCompletionKind::Function => {
                    if let Some(func) = gm_manual.functions.get(&completion.label) {
                        // compose signature:
                        let detail = format!(
                            "{}({}): {}",
                            func.name,
                            func.parameters.iter().map(|v| &v.parameter).format(", "),
                            func.returns
                        );
                        completion.detail = Some(detail);

                        let value = format!(
                            "{}\n## Examples\n{}\nGo to [{}]({})",
                            func.description, func.example, func.name, func.link
                        );

                        // gather documentation:
                        completion.documentation =
                            Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                                kind: lsp_types::MarkupKind::Markdown,
                                value,
                            }));
                    }
                }
                StdCompletionKind::Variable => {
                    if let Some(variable) = gm_manual.variables.get(&completion.label) {
                        // get some nice typing in there...
                        completion.detail =
                            Some(format!("{}: {}", variable.name, variable.returns));

                        // get some gucci documentation...but what about our links? how should those look?
                        completion.documentation =
                            Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                                kind: lsp_types::MarkupKind::Markdown,
                                value: variable.description.clone(),
                            }));
                    }
                }
                StdCompletionKind::Constant => {
                    if let Some(constant) = gm_manual.constants.get(&completion.label) {
                        {
                            completion.detail = Some(constant.name.clone());
                            completion.documentation =
                                Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                                    kind: lsp_types::MarkupKind::Markdown,
                                    value: constant.description.clone(),
                                }));

                            // secondary descriptors??
                            // link?
                        }
                    }
                }
            }
        }
    }

    completion
}

#[derive(
    Debug, Copy, Clone, Eq, Ord, PartialOrd, PartialEq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum StdCompletionKind {
    Function,
    Variable,
    Constant,
}
