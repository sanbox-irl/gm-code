use super::utils::{self, StdCompletionKind};
use crate::GmManual;
use lsp_types::{Hover, HoverContents};
use yy_boss::YypBoss;

pub fn hover_on_word(word: &str, gm_manual: &GmManual, yy_boss: &YypBoss) -> Option<Hover> {
    const INPUT: [StdCompletionKind; 4] = [
        StdCompletionKind::Function,
        StdCompletionKind::Variable,
        StdCompletionKind::Constant,
        StdCompletionKind::Object,
    ];

    utils::detailed_docs_data(word, &INPUT, gm_manual, yy_boss).map(|mut v| {
        v.description.insert(
            0,
            lsp_types::MarkedString::from_language_code("gml-gms2".to_string(), v.detail),
        );

        Hover {
            contents: HoverContents::Array(v.description),
            range: None,
        }
    })
}
