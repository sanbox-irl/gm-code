use anyhow::Result as AnyResult;
use log::info;
use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::{
    request::Completion, CompletionItem, CompletionItemKind, InitializeParams, MarkupContent,
    ServerCapabilities,
};

mod intellisense {
    pub mod completion;
    // pub use completion::*;
}

mod services {
    mod gm_docs;
    pub use gm_docs::{
        GmManual, GmManualConstant, GmManualFunction, GmManualFunctionParameter, GmManualVariable,
    };

    mod services_provider;
    pub use services_provider::ServicesProvider;
}
pub use services::*;

fn main() -> AnyResult<()> {
    flexi_logger::Logger::with_str("info, gm-code = debug")
        .start()
        .unwrap();
    info!("starting gm-code");

    let (connection, io_threads) = Connection::stdio();

    let server_capabs = ServerCapabilities {
        completion_provider: Some(lsp_types::CompletionOptions::default()),
        ..ServerCapabilities::default()
    };

    let server_capabilities = serde_json::to_value(&server_capabs).unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    let params: InitializeParams = serde_json::from_value(initialization_params).unwrap();

    main_loop(&connection, params)?;
    io_threads.join()?;

    // Shut down gracefully.
    info!("shutting down gm-code server");
    Ok(())
}

fn main_loop(connection: &Connection, _params: InitializeParams) -> AnyResult<()> {
    info!("starting main loop");
    let services = ServicesProvider::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                let request = match cast::<Completion>(req) {
                    Ok((id, params)) => {
                        info!("got completion request #{}: {:?}", id, params);
                        let result = intellisense::completion::completion()

                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(req) => req,
                };

                info!("dropped request {:?}", request);
                // ...
            }
            Message::Response(resp) => {
                info!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                info!("got notification: {:?}", not);
            }
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), Request>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
