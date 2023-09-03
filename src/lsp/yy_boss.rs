use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum YyBossRequest {}

impl lsp_types::request::Request for YyBossRequest {
    type Params = yy_boss::cli::Command;
    type Result = Option<()>;
    const METHOD: &'static str = "textDocument/yyBoss";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializationOptions {
    pub working_directory: String,
}
