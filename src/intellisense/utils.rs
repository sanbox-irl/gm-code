use itertools::Itertools;
use lsp_types::MarkedString;
use yy_boss::YypBoss;

use strum::IntoEnumIterator;

#[derive(
    Debug,
    Copy,
    Clone,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum_macros::EnumIter,
)]
pub enum StdCompletionKind {
    Function,
    Variable,
    Constant,
    Object,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DetailedDocsData {
    pub detail: String,
    pub description: Vec<MarkedString>,
}

pub fn detailed_docs_data(
    input: &str,
    attempt: &[StdCompletionKind],
    gm_manual: &gm_doc::Program,
    _yyp_boss: &YypBoss,
) -> Option<DetailedDocsData> {
    for kind in StdCompletionKind::iter() {
        if attempt.contains(&kind) {
            match kind {
                StdCompletionKind::Function => {
                    if let Some(func) = gm_manual.functions.get(input) {
                        // compose signature:
                        let detail = format!(
                            "{}({}): {}",
                            func.name,
                            func.parameters.iter().map(|v| &v.name).format(", "),
                            func.returns
                        );

                        // gather documentation:
                        let mut description =
                            vec![MarkedString::from_markdown(func.description.to_string())];
                        if let Some(link) = &func.link {
                            description.push(MarkedString::from_markdown(format!(
                                "Go to [{}]({})",
                                func.name, link
                            )));
                        }

                        return Some(DetailedDocsData {
                            detail,
                            description,
                        });
                    }
                }
                StdCompletionKind::Variable => {
                    if let Some(variable) = gm_manual.variables.get(input) {
                        let detail = format!("{}: {}", variable.name, variable.returns);

                        let mut description = vec![MarkedString::from_markdown(
                            variable.description.to_string(),
                        )];
                        if let Some(link) = &variable.link {
                            description.push(MarkedString::from_markdown(format!(
                                "Go to [{}]({})",
                                variable.name, link
                            )));
                        }

                        return Some(DetailedDocsData {
                            detail,
                            description,
                        });
                    }
                }
                StdCompletionKind::Constant => {
                    if let Some(constant) = gm_manual.constants.get(input) {
                        {
                            let detail = constant.name.clone();

                            let mut description = vec![MarkedString::from_markdown(
                                constant.description.to_string(),
                            )];
                            if let Some(link) = &constant.link {
                                description.push(MarkedString::from_markdown(format!(
                                    "Go to [{}]({})",
                                    constant.name, link
                                )));
                            }

                            return Some(DetailedDocsData {
                                detail,
                                description,
                            });
                        }
                    }
                }

                StdCompletionKind::Object => {
                    return Some(DetailedDocsData {
                        detail: input.to_string(),
                        description: vec![],
                    })
                }
            }
        }
    }

    None
}
