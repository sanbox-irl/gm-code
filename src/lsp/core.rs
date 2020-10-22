#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl From<lsp_types::Position> for Position {
    fn from(o: lsp_types::Position) -> Self {
        Self {
            line: o.line as usize,
            column: o.character as usize,
        }
    }
}
