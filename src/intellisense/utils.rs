use itertools::Itertools;
use lsp_types::MarkedString;
use yy_boss::YypBoss;

use crate::GmManual;
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
    gm_manual: &GmManual,
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
                            func.parameters.iter().map(|v| &v.parameter).format(", "),
                            func.returns
                        );

                        // gather documentation:
                        let value = format!("{}\n## Examples\n{}", func.description, func.example);

                        let description = vec![
                            MarkedString::from_markdown(value),
                            MarkedString::from_markdown(format!(
                                "Go to [{}]({})",
                                func.name, func.link
                            )),
                        ];

                        return Some(DetailedDocsData {
                            detail,
                            description,
                        });
                    }
                }
                StdCompletionKind::Variable => {
                    if let Some(variable) = gm_manual.variables.get(input) {
                        let detail = format!("{}: {}", variable.name, variable.returns);

                        let value = format!(
                            "{}\n## Examples\n{}",
                            variable.description, variable.example,
                        );

                        let description = vec![
                            MarkedString::from_markdown(value),
                            MarkedString::from_markdown(format!(
                                "Go to [{}]({})",
                                variable.name, variable.link
                            )),
                        ];

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
                            let mut value = constant.description.clone();
                            if let Some(secondary) = &constant.secondary_descriptors {
                                value.push_str(&format!(
                                    "\n{}\n",
                                    secondary
                                        .iter()
                                        .map(|(k, v)| format!("{}: {}", k, v))
                                        .format("\n")
                                ));
                            }

                            let description = vec![
                                MarkedString::from_markdown(value),
                                MarkedString::from_markdown(format!(
                                    "\nGo to [{}]({})",
                                    constant.name, constant.link
                                )),
                            ];

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
