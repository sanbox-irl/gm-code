use std::{
    iter::{Rev, Skip},
    str::Chars,
};

use itertools::Itertools;
use lsp_types::{Documentation, MarkupContent, SignatureHelp};

use crate::{GmManual, Position};

pub fn signature_help(
    document: &str,
    position: Position,
    gm_manual: &GmManual,
) -> Option<SignatureHelp> {
    func_name_and_param(document, position)
        .and_then(|(name, active_parameter)| {
            gm_manual.functions.get(&name).map(|func| {
                // compose signature:
                let label = format!(
                    "{}({}): {}",
                    func.name,
                    func.parameters.iter().map(|v| &v.parameter).format(", "),
                    func.returns
                );

                // gather documentation:
                let value = format!("{}\n## Examples\n{}\n", func.description, func.example);

                // gather parameters:
                let parameters = func
                    .parameters
                    .iter()
                    .map(|p| lsp_types::ParameterInformation {
                        label: lsp_types::ParameterLabel::Simple(p.parameter.to_string()),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: lsp_types::MarkupKind::Markdown,
                            value: p.description.to_string(),
                        })),
                    })
                    .collect();

                lsp_types::SignatureInformation {
                    label,
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value,
                    })),
                    parameters: Some(parameters),
                    active_parameter: Some(active_parameter as i64),
                }
            })
        })
        .map(|signature_information| SignatureHelp {
            active_parameter: signature_information.active_parameter,
            signatures: vec![signature_information],
            active_signature: Some(0),
        })
}

fn func_name_and_param(document: &str, position: Position) -> Option<(String, usize)> {
    get_pos_in_document(document, position).and_then(|pos| {
        log::info!("we're in the document...");
        let mut iter = SignatureIterator::new(document, pos);

        iter.eat_parameters().and_then(|count| {
            log::info!("parameters have been eaten");
            iter.eat_identifier().map(|ident| (ident, count))
        })
    })
}

pub fn get_pos_in_document(document: &str, pos: Position) -> Option<usize> {
    let mut lines_to_go = pos.line;
    let mut offset = if lines_to_go == 0 { Some(0) } else { None };

    for (i, chr) in document.char_indices() {
        if let Some(offset) = offset {
            if i - offset == pos.column {
                return Some(i);
            }
        }

        if chr == '\n' {
            if lines_to_go == 0 {
                return None;
            } else {
                lines_to_go -= 1;
                if lines_to_go == 0 {
                    offset = Some(i + 1);
                }
            }
        }
    }

    None
}

struct SignatureIterator<'a> {
    iter: Skip<Rev<Chars<'a>>>,
}

impl<'a> SignatureIterator<'a> {
    pub fn new(data: &'a str, start: usize) -> Self {
        Self {
            iter: data.chars().rev().skip(data.len() - start),
        }
    }

    pub fn eat_parameters(&mut self) -> Option<usize> {
        let mut paren_nesting: usize = 0;
        let mut bracket_nesting: usize = 0;
        let mut param_count: usize = 0;

        while let Some(n) = self.iter.next() {
            match n {
                '(' => {
                    if paren_nesting == 0 {
                        return Some(param_count);
                    } else {
                        paren_nesting -= 1;
                    }
                }
                ')' => {
                    paren_nesting += 1;
                }
                '[' => {
                    if bracket_nesting != 0 {
                        bracket_nesting -= 1;
                    }
                }
                ']' => {
                    bracket_nesting += 1;
                }
                '"' | '\'' => {
                    // NOMM until we find the other pair
                    for sub in &mut self.iter {
                        if n == sub {
                            break;
                        }
                    }
                }
                ',' => {
                    // add the GLORIOUS param count
                    if paren_nesting == 0 && bracket_nesting == 0 {
                        param_count += 1;
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub fn eat_identifier(self) -> Option<String> {
        let mut ident_started = false;
        let mut identifier = Vec::new();

        for chr in self.iter {
            if ident_started == false && chr.is_whitespace() {
                continue;
            }

            if chr.is_alphanumeric() || chr == '_' {
                ident_started = true;

                identifier.push(chr);
            } else if ident_started {
                break;
            }
        }

        if identifier.is_empty() {
            None
        } else {
            identifier.reverse();
            Some(identifier.into_iter().collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn char_pos_from_string(input: &str) -> (usize, String) {
        let pos = input
            .char_indices()
            .find_map(|v| if v.1 == '?' { Some(v.0) } else { None })
            .unwrap();

        let mut input = input.to_string();
        input.remove(pos);

        (pos, input)
    }

    #[test]
    fn harness_tests() {
        let (idx, input) = char_pos_from_string("show_debug_message(?x, y);");
        assert_eq!(idx, 19);
        let position = Position::new_idx(idx, &input);
        assert_eq!(
            position,
            Position {
                line: 0,
                column: 19
            }
        );

        let (idx, input) = char_pos_from_string("show_debug_message\r\n\t(?x, y);");
        assert_eq!(idx, 22);
        let position = Position::new_idx(idx, &input);
        assert_eq!(position, Position { line: 1, column: 2 });
    }

    #[test]
    fn signature_iterator() {
        let (idx, input) = char_pos_from_string("show_debug_message(x,? y);");
        let input = &input;

        let mut sig = SignatureIterator::new(input, idx);
        assert_eq!(1, sig.eat_parameters().unwrap());
        assert_eq!("show_debug_message", sig.eat_identifier().unwrap());

        let mut sig = SignatureIterator::new(input, idx - 1);
        assert_eq!(0, sig.eat_parameters().unwrap());
        assert_eq!("show_debug_message", sig.eat_identifier().unwrap());

        let mut sig = SignatureIterator::new(input, idx - 2);
        assert_eq!(0, sig.eat_parameters().unwrap());
        assert_eq!("show_debug_message", sig.eat_identifier().unwrap());

        let mut sig = SignatureIterator::new(input, idx - 3);
        assert!(sig.eat_parameters().is_none());
        assert!(sig.eat_identifier().is_none());
    }

    #[test]
    fn full() {
        let (idx, input) = char_pos_from_string("show_debug_message(x,? y);");
        let position = Position::new_idx(idx, &input);

        assert_eq!(
            ("show_debug_message".to_string(), 1),
            func_name_and_param(&input, position).unwrap()
        );

        let (idx, input) = char_pos_from_string("show_debug_message(x?, y);");
        let position = Position::new_idx(idx, &input);

        assert_eq!(
            ("show_debug_message".to_string(), 0),
            func_name_and_param(&input, position).unwrap()
        );

        let (idx, input) = char_pos_from_string("show_debug_message(x, y?);");
        let position = Position::new_idx(idx, &input);

        assert_eq!(
            ("show_debug_message".to_string(), 1),
            func_name_and_param(&input, position).unwrap()
        );

        let (idx, input) = char_pos_from_string("show_debug_message(\n?x);");
        let position = Position::new_idx(idx, &input);

        assert_eq!(
            ("show_debug_message".to_string(), 0),
            func_name_and_param(&input, position).unwrap()
        );

        let (idx, input) = char_pos_from_string("show_debug_message(\n,?x);");
        let position = Position::new_idx(idx, &input);

        assert_eq!(
            ("show_debug_message".to_string(), 1),
            func_name_and_param(&input, position).unwrap()
        );

        let (idx, input) = char_pos_from_string("show_debug_message(,,?);");
        let position = Position::new_idx(idx, &input);

        assert_eq!(
            ("show_debug_message".to_string(), 2),
            func_name_and_param(&input, position).unwrap()
        );

        let (idx, input) = char_pos_from_string("warn(\"this is a message, yup {}\"?, y);");
        let position = Position::new_idx(idx, &input);

        assert_eq!(
            ("warn".to_string(), 0),
            func_name_and_param(&input, position).unwrap()
        );
    }
}
