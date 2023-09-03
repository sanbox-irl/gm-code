use gm_doc::Program;

const DOCS_TEXT: &str = include_str!("../../docs.json");

#[derive(Debug)]
pub struct ServicesProvider {
    gm_manual: Program,
}

impl ServicesProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gm_manual(&self) -> &Program {
        &self.gm_manual
    }
}

impl Default for ServicesProvider {
    fn default() -> Self {
        Self {
            gm_manual: serde_json::from_str(DOCS_TEXT).unwrap(),
        }
    }
}
