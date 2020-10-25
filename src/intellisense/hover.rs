use super::utils::{self, StdCompletionKind};
use crate::GmManual;
use lsp_types::{Hover, HoverContents};

pub fn hover_on_word(word: &str, gm_manual: &GmManual) -> Option<Hover> {
    const INPUT: [StdCompletionKind; 3] = [
        StdCompletionKind::Function,
        StdCompletionKind::Variable,
        StdCompletionKind::Constant,
    ];

    utils::detailed_docs_data(word, &INPUT, gm_manual).map(|mut v| {
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
