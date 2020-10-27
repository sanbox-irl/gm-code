use std::{
    iter::{Rev, Skip},
    str::Chars,
};

use log::info;
use lsp_types::SignatureHelp;

use crate::{GmManual, Position};

pub fn signature_help(
    document: &str,
    position: Position,
    gm_manual: &GmManual,
) -> Option<SignatureHelp> {
    info!("pos is {:?}", position);

    get_pos_in_document(document, position).and_then(|pos| {
        let mut iter = SignatureIterator::new(document, pos);

        iter.eat_parameters().and_then(|count| {
            iter.eat_identifier().map(|ident| {
                info!("{} at param {}", ident, count);
                1
            });

            None
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
                    offset = Some(i);
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
                    loop {
                        if let Some(sub) = self.iter.next() {
                            if n == sub {
                                break;
                            }
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
    #[test]
    pub fn test() {
        let input = "show_debug_message(x, y);";
        let idx = 21;

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
}
