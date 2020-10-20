use super::gm_docs::{create_manual, GmManual};

#[derive(Debug)]
pub struct ServicesProvider {
    gm_manual: GmManual,
}

impl ServicesProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gm_manual(&self) -> &GmManual {
        &self.gm_manual
    }
}

impl Default for ServicesProvider {
    fn default() -> Self {
        Self {
            gm_manual: create_manual(),
        }
    }
}
