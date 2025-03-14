use std::time::Duration;

use log::info;
use serde_json::Value;

use crate::{context::ServerContextSnapshot, util::time_cancel_token};
use emmylua_code_analysis::file_path_to_uri;
use lsp_types::Uri;

use super::ClientConfig;

async fn scope_uri_from_context(context: &ServerContextSnapshot) -> Option<Uri> {
    let workspace_folders = context
        .workspace_manager
        .read()
        .await
        .workspace_folders
        .clone();
    let main_workspace_folder = workspace_folders.first()?;
    let scope_uri = file_path_to_uri(main_workspace_folder);
    scope_uri
}

pub async fn get_client_config_neovim(
    context: &ServerContextSnapshot,
    config: &mut ClientConfig,
) -> Option<()> {
    let client = &context.client;
    let params = lsp_types::ConfigurationParams {
        items: vec![lsp_types::ConfigurationItem {
            scope_uri: scope_uri_from_context(context).await,
            section: Some("Lua".to_string()),
        }],
    };
    let cancel_token = time_cancel_token(Duration::from_secs(5));
    let configs = client
        .get_configuration::<Value>(params, cancel_token)
        .await?
        .into_iter()
        .filter(|config| !config.is_null())
        .collect();

    if let Some(pretty_json) = serde_json::to_string_pretty(&configs).ok() {
        info!("load neovim client config: {}", pretty_json);
    }

    config.partial_emmyrcs = Some(configs);

    Some(())
}
