#![allow(clippy::bool_comparison)]

use anyhow::Result as AnyResult;
use log::info;
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument},
    request::{Completion, HoverRequest, ResolveCompletionItem, SignatureHelpRequest},
    CompletionList, Hover, InitializeParams, ServerCapabilities, SignatureHelp,
    SignatureHelpOptions, WorkDoneProgressOptions,
};

mod intellisense {
    pub mod completion;
    pub mod hover;
    pub mod signature;
    mod utils;
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

    mod yy_boss;
    pub use self::yy_boss::*;
}
pub use lsp::*;
use yy_boss::cli::yy_cli::YyCli;

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
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
            retrigger_characters: None,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
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
    let mut boss = Boss::new(&params.root_uri.unwrap());
    let initialization_options: InitializationOptions =
        serde_json::from_value(params.initialization_options.unwrap()).unwrap();
    let yy_cli =
        YyCli::new(std::path::Path::new(&initialization_options.working_directory).to_owned());

    for msg in &connection.receiver {
        match msg {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }

                let request = match cast::<Completion>(request) {
                    Ok((id, params)) => {
                        info!("received completion requestion msg {}: {:?}", id, params);
                        let position = params.text_document_position.position;

                        let result: CompletionList = boss
                            .get_text_document(&params.text_document_position.text_document.uri)
                            .and_then(|v| {
                                Boss::get_word_in_document(v, position).map(|word| {
                                    intellisense::completion::initial_completion(
                                        word,
                                        services.gm_manual(),
                                        &boss.yy_boss,
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
                            "got resolve completion request #{}: {:?}",
                            id, completion_item
                        );
                        let completion_item = intellisense::completion::resolve_completion(
                            completion_item,
                            services.gm_manual(),
                            &boss.yy_boss,
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

                let request = match cast::<HoverRequest>(request) {
                    Ok((id, params)) => {
                        info!("got hover request #{}: {:?}", id, params);
                        let position = params.text_document_position_params;

                        let result: Option<Hover> = boss
                            .get_text_document(&position.text_document.uri)
                            .and_then(|v| {
                                Boss::get_word_in_document_full(v, position.position).and_then(
                                    |word| {
                                        intellisense::hover::hover_on_word(
                                            word,
                                            services.gm_manual(),
                                            &boss.yy_boss,
                                        )
                                    },
                                )
                            });

                        let resp = Response {
                            id,
                            result: Some(
                                result
                                    .map(|v| serde_json::to_value(&v).unwrap())
                                    .unwrap_or(serde_json::Value::Null),
                            ),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;

                        continue;
                    }
                    Err(r) => r,
                };

                let request = match cast::<SignatureHelpRequest>(request) {
                    Ok((id, params)) => {
                        info!("got signature request #{}: {:?}", id, params);
                        let result: Option<SignatureHelp> = boss
                            .get_text_document(
                                &params.text_document_position_params.text_document.uri,
                            )
                            .and_then(|txt| {
                                intellisense::signature::signature_help(
                                    txt,
                                    params.text_document_position_params.position.into(),
                                    services.gm_manual(),
                                )
                            });

                        let resp = Response {
                            id,
                            result: Some(
                                result
                                    .map(|v| serde_json::to_value(&v).unwrap())
                                    .unwrap_or(serde_json::Value::Null),
                            ),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;

                        continue;
                    }
                    Err(r) => r,
                };

                let request = match cast::<lsp::YyBossRequest>(request) {
                    Ok((id, param)) => {
                        let mut shutdown = false;
                        let output = yy_cli.parse_command(param, &mut boss.yy_boss, &mut shutdown);

                        let resp = Response {
                            id,
                            result: Some(serde_json::to_value(&output).unwrap()),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;

                        continue;
                    }
                    Err(e) => e,
                };

                info!("dropped request {:?}", request);
                // ...
            }
            Message::Response(_resp) => {
                // info!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                let not = match cast_notification::<DidOpenTextDocument>(not) {
                    Ok(v) => {
                        if let Some(txt) = boss.get_text_document_mut(&v.text_document.uri) {
                            *txt = v.text_document.text;
                        }
                        continue;
                    }
                    Err(e) => e,
                };

                let not = match cast_notification::<DidChangeTextDocument>(not) {
                    Ok(v) => {
                        // info!("got didchangetextdocument: {:?}", v);
                        if let Some(txt) = boss.get_text_document_mut(&v.text_document.uri) {
                            for change in v.content_changes {
                                if let Some(range) = change.range {
                                    let range: Range = range.into();
                                    let start = range.start.get_idx(txt).unwrap();
                                    let end = range.end.get_idx(txt).unwrap();

                                    txt.replace_range(start..end, &change.text);
                                } else {
                                    *txt = change.text;
                                }
                            }
                        }

                        continue;
                    }
                    Err(e) => e,
                };

                let _not = match cast_notification::<DidSaveTextDocument>(not) {
                    Ok(v) => {
                        // info!("got didchangetextdocument: {:?}", v);
                        if let Some(txt) = boss.get_text_document_mut(&v.text_document.uri) {
                            *txt = v.text.unwrap();
                        }
                        continue;
                    }
                    Err(e) => e,
                };

                // info!("got notification: {:?}", not);
            }
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), Request>
where
    R: lsp_types::request::Request,
{
    req.extract(R::METHOD)
}

fn cast_notification<N>(req: Notification) -> Result<N::Params, Notification>
where
    N: lsp_types::notification::Notification,
{
    req.extract(N::METHOD)
}
