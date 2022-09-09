use super::utils;
use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, MarkedString, MarkupContent};
use yy_boss::YypBoss;

use crate::services::GmManual;

use super::utils::StdCompletionKind;

pub fn initial_completion(
    input_str: &str,
    gm_manual: &GmManual,
    yy_boss: &YypBoss,
) -> CompletionList {
    let mut output = vec![];

    // check for functions:
    for func in gm_manual.functions.values() {
        if func.name.contains(input_str) {
            output.push(CompletionItem {
                label: func.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
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
                kind: Some(CompletionItemKind::VARIABLE),
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
                kind: Some(CompletionItemKind::VALUE),
                data: serde_json::to_value(StdCompletionKind::Constant).ok(),

                ..CompletionItem::default()
            })
        }
    }

    // Check for object names:
    for obj_name in yy_boss.objects.into_iter() {
        if obj_name.yy_resource.common_data.name.contains(input_str) {
            output.push(CompletionItem {
                label: obj_name.yy_resource.common_data.name.clone(),
                kind: Some(CompletionItemKind::CONSTRUCTOR),
                data: serde_json::to_value(StdCompletionKind::Object).ok(),

                ..CompletionItem::default()
            })
        }
    }

    for sprite_name in yy_boss.sprites.into_iter() {
        if sprite_name.yy_resource.common_data.name.contains(input_str) {
            output.push(CompletionItem {
                label: sprite_name.yy_resource.common_data.name.clone(),
                kind: Some(CompletionItemKind::COLOR),
                data: serde_json::to_value(StdCompletionKind::Object).ok(),

                ..CompletionItem::default()
            })
        }
    }

    for shader_name in yy_boss.shaders.into_iter() {
        if shader_name.yy_resource.common_data.name.contains(input_str) {
            output.push(CompletionItem {
                label: shader_name.yy_resource.common_data.name.clone(),
                kind: Some(CompletionItemKind::COLOR),
                data: serde_json::to_value(StdCompletionKind::Object).ok(),

                ..CompletionItem::default()
            })
        }
    }

    CompletionList {
        is_incomplete: true,
        items: output,
    }
}

pub fn resolve_completion(
    mut completion: CompletionItem,
    gm_manual: &GmManual,
    yy_boss: &YypBoss,
) -> CompletionItem {
    if let Some(data) = completion.data.clone() {
        if let Ok(v) = serde_json::from_value(data) {
            if let Some(output) =
                utils::detailed_docs_data(&completion.label, &[v], gm_manual, yy_boss)
            {
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
