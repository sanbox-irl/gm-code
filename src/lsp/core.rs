use log::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn get_idx(self, txt: &str) -> Option<usize> {
        let mut lines_to_go = self.line;
        let mut chars_to_go = self.column;

        for (i, chr) in txt.char_indices() {
            if lines_to_go == 0 && chars_to_go == 0 {
                return Some(i);
            }

            if chr == '\n' {
                if lines_to_go == 0 {
                    return None;
                }
                lines_to_go -= 1;
            } else if lines_to_go == 0 {
                chars_to_go -= 1;
            }
        }

        if lines_to_go == 0 && chars_to_go == 0 {
            Some(txt.len())
        } else {
            info!("gonna crash, at {} and {}", lines_to_go, chars_to_go);
            None
        }
    }
}

#[cfg(test)]
impl Position {
    pub fn new_idx(char_idx: usize, input: &str) -> Self {
        let mut column = 0;
        let mut line = 0;

        for (i, chr) in input.char_indices() {
            if i == char_idx {
                break;
            }

            match chr {
                '\n' => {
                    line += 1;
                    column = 0;
                }
                '\r' => {}
                _ => {
                    column += 1;
                }
            }
        }

        Self { line, column }
    }
}

impl From<lsp_types::Position> for Position {
    fn from(o: lsp_types::Position) -> Self {
        Self {
            line: o.line as usize,
            column: o.character as usize,
        }
    }
}

impl From<lsp_types::Range> for Range {
    fn from(o: lsp_types::Range) -> Self {
        Range {
            start: o.start.into(),
            end: o.end.into(),
        }
    }
}
