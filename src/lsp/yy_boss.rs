use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum YyBossRequest {}

impl lsp_types::request::Request for YyBossRequest {
    type Params = YyBossRequestParams;
    type Result = Option<()>;
    const METHOD: &'static str = "textDocument/yyBoss";
}

#[derive(Debug, Serialize, Deserialize)]
pub enum YyBossRequestParams {
    HelloWorld,
}
