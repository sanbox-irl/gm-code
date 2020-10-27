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

        iter.read_arguments().and_then(|count| {
            let ident = iter.read_func_identifier();

            if ident.is_empty() {
                return None;
            }

            info!("{} at param {}", ident, count);

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
    start: usize,
    data: &'a str,
}

impl<'a> SignatureIterator<'a> {
    pub fn new(data: &'a str, start: usize) -> Self {
        Self {
            iter: data.chars().rev().skip(data.len() - start),
            start,
            data,
        }
    }

    pub fn read_arguments(&mut self) -> Option<usize> {
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

    pub fn read_func_identifier(self) -> String {
        let mut ident_started = false;
        let mut identifier = vec![];

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

        identifier.reverse();
        identifier.into_iter().collect()
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
        assert_eq!(1, sig.read_arguments().unwrap());
        assert_eq!("show_debug_message", sig.read_func_identifier());

        let mut sig = SignatureIterator::new(input, idx - 1);
        assert_eq!(0, sig.read_arguments().unwrap());
        assert_eq!("show_debug_message", sig.read_func_identifier());

        let mut sig = SignatureIterator::new(input, idx - 2);
        assert_eq!(0, sig.read_arguments().unwrap());
        assert_eq!("show_debug_message", sig.read_func_identifier());

        let mut sig = SignatureIterator::new(input, idx - 3);
        assert!(sig.read_arguments().is_none());
        assert!(sig.read_func_identifier().is_empty());
    }
}
