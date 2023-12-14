#![allow(clippy::bool_comparison)]

use anyhow::Result as AnyResult;
use log::info;
use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument},
    request::{Completion, HoverRequest, ResolveCompletionItem, SignatureHelpRequest},
    CompletionList, Hover, InitializeParams, ServerCapabilities, SignatureHelp,
    SignatureHelpOptions, WorkDoneProgressOptions,
};

mod intellisense;
use intellisense::*;

mod services;
use services::{Boss, ServicesProvider};

mod lsp;

const EXTENSION_NEEDLE: Option<&str> = Some("yyp");

fn main() -> AnyResult<()> {
    flexi_logger::Logger::try_with_str("info")
        .unwrap()
        .start()
        .unwrap();
    info!("starting gm-code");

    let (connection, io_threads) = Connection::stdio();

    let server_capabs = ServerCapabilities {
        text_document_sync: Some(
            lsp_types::TextDocumentSyncOptions {
                change: Some(lsp_types::TextDocumentSyncKind::INCREMENTAL),
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

    let server_capabilities = serde_json::to_value(server_capabs).unwrap();
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
    let final_path =
        params
            .workspace_folders
            .unwrap()
            .into_iter()
            .find_map(|wrkspace_folder_path| {
                let file = wrkspace_folder_path.uri.to_file_path().ok()?;
                let file = camino::Utf8PathBuf::from_path_buf(file).ok()?;

                if file.is_file() {
                    (file.extension() == EXTENSION_NEEDLE).then_some(file)
                } else {
                    // we got a folder, which makes plenty of sense. let's see if there's a
                    // yyp in here...
                    file.read_dir()
                        .ok()?
                        .filter_map(|v| v.ok())
                        .find_map(|dir_entry| {
                            let path = camino::Utf8PathBuf::from_path_buf(dir_entry.path()).ok()?;
                            (path.extension() == EXTENSION_NEEDLE).then_some(path)
                        })
                }
            });

    let Some(final_path) = final_path else {
        info!("No .yyp file found. Exiting...");
        return Ok(());
    };

    let mut boss = Boss::new(final_path);
    let initialization_options: lsp::InitializationOptions =
        serde_json::from_value(params.initialization_options.unwrap()).unwrap();

    let working_directory = camino::Utf8PathBuf::from(&initialization_options.working_directory);

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
                                    completion::initial_completion(
                                        word,
                                        services.gm_manual(),
                                        &boss.yy_boss,
                                    )
                                })
                            })
                            .unwrap_or_default();

                        let result = serde_json::to_value(result).unwrap();
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
                        let completion_item = completion::resolve_completion(
                            completion_item,
                            services.gm_manual(),
                            &boss.yy_boss,
                        );

                        let result = serde_json::to_value(completion_item).unwrap();
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

                let request = match cast::<HoverRequest>(request) {
                    Ok((id, params)) => {
                        let position = params.text_document_position_params;

                        let result: Option<Hover> = boss
                            .get_text_document(&position.text_document.uri)
                            .and_then(|v| {
                                Boss::get_word_in_document_full(v, position.position).and_then(
                                    |word| {
                                        hover::hover_on_word(
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
                                    .map(|v| serde_json::to_value(v).unwrap())
                                    .unwrap_or(serde_json::Value::Null),
                            ),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;

                        continue;
                    }
                    Err(req) => req,
                };

                let request = match cast::<SignatureHelpRequest>(request) {
                    Ok((id, params)) => {
                        let result: Option<SignatureHelp> = boss
                            .get_text_document(
                                &params.text_document_position_params.text_document.uri,
                            )
                            .and_then(|txt| {
                                signature::signature_help(
                                    txt,
                                    params.text_document_position_params.position.into(),
                                    services.gm_manual(),
                                )
                            });

                        let resp = Response {
                            id,
                            result: Some(
                                result
                                    .map(|v| serde_json::to_value(v).unwrap())
                                    .unwrap_or(serde_json::Value::Null),
                            ),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;

                        continue;
                    }
                    Err(req) => req,
                };

                match cast::<lsp::YyBossRequest>(request) {
                    Ok((id, param)) => {
                        let output = yy_boss::cli::parse_command(
                            param,
                            &working_directory,
                            &mut boss.yy_boss,
                        );

                        let resp = Response {
                            id,
                            result: Some(serde_json::to_value(output).unwrap()),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;

                        continue;
                    }
                    Err(req) => req,
                };
            }
            Message::Response(_resp) => {}
            Message::Notification(not) => {
                let not = match cast_notification::<DidOpenTextDocument>(not) {
                    Ok(v) => {
                        if let Some(txt) = boss.get_text_document_mut(&v.text_document.uri) {
                            *txt = v.text_document.text;
                        }
                        continue;
                    }
                    Err(req) => req,
                };

                let not = match cast_notification::<DidChangeTextDocument>(not) {
                    Ok(v) => {
                        if let Some(txt) = boss.get_text_document_mut(&v.text_document.uri) {
                            for change in v.content_changes {
                                if let Some(range) = change.range {
                                    let range: lsp::Range = range.into();
                                    let start = range.start.get_idx(txt).unwrap();
                                    let end = range.end.get_idx(txt).unwrap();

                                    txt.replace_range(start..end, &change.text);
                                } else {
                                    *txt = change.text;
                                }
                            }
                        } else {
                            log::warn!("text_document.uri not found {}", v.text_document.uri);
                        }

                        continue;
                    }
                    Err(req) => req,
                };

                let _not = match cast_notification::<DidSaveTextDocument>(not) {
                    Ok(v) => {
                        if let Some(txt) = boss.get_text_document_mut(&v.text_document.uri) {
                            *txt = v.text.unwrap();
                        }
                        continue;
                    }
                    Err(e) => e,
                };
            }
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), Request>
where
    R: lsp_types::request::Request,
{
    match req.extract::<R::Params>(R::METHOD) {
        Ok(v) => Ok(v),
        Err(e) => match e {
            ExtractError::MethodMismatch(input) => Err(input),
            ExtractError::JsonError { .. } => {
                panic!("extraction error: {}", e);
            }
        },
    }
}

fn cast_notification<N>(req: Notification) -> Result<N::Params, Notification>
where
    N: lsp_types::notification::Notification,
{
    match req.extract(N::METHOD) {
        Ok(v) => Ok(v),
        Err(e) => match e {
            ExtractError::MethodMismatch(input) => Err(input),
            ExtractError::JsonError { .. } => {
                panic!("extraction error: {}", e);
            }
        },
    }
}
