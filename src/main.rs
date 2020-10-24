use anyhow::Result as AnyResult;
use log::info;
use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::{
    request::{Completion, ResolveCompletionItem},
    CompletionList, InitializeParams, ServerCapabilities,
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
    mod boss;
    pub use boss::Boss;

    mod services_provider;
    pub use services_provider::ServicesProvider;
}
pub use services::*;

mod lsp {
    mod core;
    pub use self::core::*;
}
pub use lsp::*;

fn main() -> AnyResult<()> {
    flexi_logger::Logger::with_str("info, gm-code = debug")
        .start()
        .unwrap();
    info!("starting gm-code");

    let (connection, io_threads) = Connection::stdio();

    let server_capabs = ServerCapabilities {
        text_document_sync: Some(
            lsp_types::TextDocumentSyncOptions {
                change: Some(lsp_types::TextDocumentSyncKind::Incremental),
                save: Some(
                    lsp_types::SaveOptions {
                        include_text: Some(true),
                    }
                    .into(),
                ),
                ..Default::default()
            }
            .into(),
        ),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(true),
            ..Default::default()
        }),
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

fn main_loop(connection: &Connection, params: InitializeParams) -> AnyResult<()> {
    info!("starting main loop");
    let services = ServicesProvider::new();
    let boss = Boss::new(&params.root_uri.unwrap());

    for msg in &connection.receiver {
        match msg {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }

                let request = match cast::<Completion>(request) {
                    Ok((id, params)) => {
                        info!("got completion request #{}: {:?}", id, params);
                        let position = params.text_document_position.position;

                        let result: CompletionList = boss
                            .get_text_document(&params.text_document_position.text_document.uri)
                            .and_then(|v| {
                                Boss::get_word_in_document(v, position).map(|word| {
                                    info!("word is {}", word);
                                    intellisense::completion::initial_completion(
                                        word,
                                        services.gm_manual(),
                                    )
                                })
                            })
                            .unwrap_or_default();

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

                let request = match cast::<ResolveCompletionItem>(request) {
                    Ok((id, completion_item)) => {
                        info!(
                            "got completion resolve request #{}: {:?}",
                            id, completion_item
                        );
                        let completion_item = intellisense::completion::resolve_completion(
                            completion_item,
                            services.gm_manual(),
                        );

                        let result = serde_json::to_value(&completion_item).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;

                        continue;
                    }
                    Err(request) => request,
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
