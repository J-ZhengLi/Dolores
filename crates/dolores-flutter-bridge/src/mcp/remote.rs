use super::*;
use dolores_core::{
    McpRemoteAuthentication, McpRemoteTarget, McpTool, MCP_REMOTE_BEARER, MCP_REMOTE_OAUTH,
};
use dolores_tools_mcp::remote::{
    credentials::{resolve_bearer, BearerCredential},
    oauth::{
        self,
        vault::{OAuthCredential, SignInConfiguration},
        LoopbackCallback, PendingAuthorization,
    },
    RemoteClient,
};
#[cfg(test)]
mod tests;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemoteInput {
    #[serde(default)]
    pub bearer: Option<String>,
    #[serde(default)]
    pub sign_in: Option<SignInConfiguration>,
}

pub(crate) struct RemoteReview {
    pub(super) token: String,
    pub(super) connection_id: String,
    session: String,
    root: String,
    previous: Option<McpConnection>,
    created: Instant,
    target: McpRemoteTarget,
    credential: Option<BearerCredential>,
    oauth: Option<OAuthCredential>,
    protocol: String,
    server_name: String,
    server_version: String,
    tools: Vec<McpTool>,
}
impl Engine {
    pub(crate) fn inspect_remote_mcp(
        &self,
        active: &mut crate::run_journal::RunCoordinator,
        id: u64,
        session: String,
        connection_id: String,
        target: McpRemoteTarget,
        input: RemoteInput,
    ) -> Result<Value, String> {
        let RemoteInput { bearer, sign_in } = input;
        target.validate()?;
        if let Some(config) = &sign_in {
            config.validate()?;
        }
        if (target.authentication != McpRemoteAuthentication::Oauth && sign_in.is_some())
            || (target.authentication != McpRemoteAuthentication::Bearer && bearer.is_some())
        {
            return Err("Choose authentication fields matching this connection.".into());
        }
        let root = self.mcp_root(&session)?;
        let connection_id = if connection_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            connection_id
        };
        if connection_id == "legacy" || !dolores_core::valid_mcp_id(&connection_id) {
            return Err(
                "Remote MCP connection identity is invalid. Add a remote connection.".into(),
            );
        }
        let all = self.store.mcp_connections(&root)?;
        let previous = all.iter().find(|c| c.id == connection_id).cloned();
        if previous.as_ref().is_some_and(|c| c.remote.is_none()) {
            return Err("Choose a remote connection or add a new endpoint connection.".into());
        }
        if previous.is_none() && all.len() >= dolores_core::MAX_MCP_CONNECTIONS {
            return Err("This folder already has four MCP connections. Forget a saved server before adding another.".into());
        }
        if target.authentication == McpRemoteAuthentication::None && bearer.is_some() {
            return Err("Remove the bearer value when no authentication is selected.".into());
        }
        self.clear_mcp_review()?;
        let cancel = CancellationToken::new();
        let (output, events) = mpsc::channel(4);
        active.reserve(Run {
            thread: None,
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        })?;
        let (store, vault, slot) = (
            self.store.clone(),
            self.mcp_credentials.clone(),
            self.mcp_remote_review.clone(),
        );
        let browser = self.mcp_browser.clone();
        self.runtime.spawn(async move {
            let result=async {
                let (saved,check_root,check_target)=(previous.clone(),root.clone(),target.clone());
                let oauth=if target.authentication==McpRemoteAuthentication::Oauth {
                    if let Some(configuration)=sign_in {
                        let listener=LoopbackCallback::bind().await.map_err(|e|e.to_string())?;
                        let mut operation=oauth::Operation::new(cancel.clone());
                        let resource=operation.discover_mcp_resource(&target.endpoint).await.map_err(|e|e.to_string())?;
                        let issuer=configuration.issuer.as_deref().or_else(||(resource.issuers().len()==1).then(||resource.issuers()[0].as_str())).ok_or("This endpoint advertises multiple sign-in issuers. Enter the intended issuer in Sign-in options.")?;
                        let server=operation.discover_server(&resource,issuer).await.map_err(|e|e.to_string())?;
                        let client=operation.register_client(&server,listener.redirect_uri(),&configuration.client_configuration()).await.map_err(|e|e.to_string())?;
                        let pending=PendingAuthorization::new(server.clone(),client.clone()).map_err(|e|e.to_string())?;
                        if cancel.is_cancelled(){return Err("MCP sign-in stopped. Nothing was saved.".into());}
                        // Browser launch is host-owned; the URL never crosses a DTO.
                        browser(pending.authorization_url())?;
                        let tokens=listener.complete(pending,cancel.clone()).await.map_err(|e|e.to_string())?;
                        Some(OAuthCredential::new(&resource,&server,&client,tokens)?)
                    } else {
                        let saved=previous.clone().ok_or("Use Sign in to connect this endpoint first.")?;
                        if saved.remote.as_ref()!=Some(&target){return Err("Remote login target changed. Use Sign in again before inspecting.".into());}
                        Some(oauth::vault::resolve(Path::new(&root).to_path_buf(),saved,vault.clone(),cancel.clone()).await?)
                    }
                } else {None};
                let deadline=tokio::time::Instant::now()+Duration::from_secs(30);
                let credential=tokio::select! {
                    biased;
                    _=cancel.cancelled()=>return Err("MCP inspection stopped. Nothing was saved.".into()),
                    _=tokio::time::sleep_until(deadline)=>return Err("MCP inspection exceeded its 30-second limit. The saved connection is unchanged; inspect again explicitly.".into()),
                    result=crate::blocking(move||match check_target.authentication {
                    McpRemoteAuthentication::None=>Ok(None),
                    McpRemoteAuthentication::Oauth=>Ok(None),
                    McpRemoteAuthentication::Bearer=>{
                        let credential=if let Some(value)=bearer{BearerCredential::new(value)?}else{
                            let saved=saved.ok_or(credentials::VAULT_ERROR)?;
                            if saved.remote.as_ref()!=Some(&check_target){return Err("Remote MCP target changed. Enter the key again before inspecting this endpoint.".into());}
                            resolve_bearer(Path::new(&check_root),&saved,vault.as_ref())?
                        };
                        credential.validate_target(&check_target)?;Ok(Some(credential))
                    },
                    })=>result?,
                };
                if cancel.is_cancelled(){return Err("MCP inspection stopped. Nothing was saved.".into());}
                let mut client=if let Some(login)=&oauth { login.connect(&target,None,cancel.clone(),deadline).await? }
                    else { RemoteClient::connect_target_until(&target,credential.as_ref(),None,cancel.clone(),deadline).await.map_err(|e|e.to_string())? };
                let discovered=client.list_tools().await;
                let server=client.server().clone();
                let excluded=client.excluded.iter().map(|t|json!({"name":t.name,"reason":t.reason})).collect::<Vec<_>>();
                let cleanup=client.close().await;
                let tools=discovered.map_err(|e|e.to_string())?;
                if tools.is_empty(){return Err("This endpoint has no supported ordinary MCP tools. Inspect another endpoint or ask the server owner to offer supported tools.".into());}
                if store.workspace(&session)?.root.as_ref()!=Some(&root) || store.mcp_connection_by_id(&root,&connection_id)?!=previous{return Err(STALE.into());}
                let token=uuid::Uuid::new_v4().to_string();
                let name=server.name.unwrap_or_else(||target.label.clone());let version=server.version.unwrap_or_default();
                let mut response=json!({"token":token,"connectionId":connection_id,"remote":target,"tools":tools,"serverName":name,"serverVersion":version,"protocolVersion":server.protocol_version,"excludedTools":excluded});
                if oauth.as_ref().is_some_and(|c|c.contains_secret(&response)){return Err("MCP metadata contains login credentials. Ask the server owner to fix it, then Sign in again.".into());}
                if cleanup.is_err(){response["warning"]=json!("Tools inspected, but remote session cleanup was not confirmed. Inspect connection health; do not repeat a tool call for cleanup.");}
                let mut review=slot.lock().map_err(|_|STALE)?;
                if cancel.is_cancelled(){return Err("MCP inspection stopped. Nothing was saved.".into());}
                *review=Some(RemoteReview{token,connection_id,session,root,previous,created:Instant::now(),target,credential,oauth,protocol:server.protocol_version,server_name:name,server_version:version,tools});
                Ok::<_,String>(response)
            }.await;
            let event=match result{Ok(value)=>json!({"type":"done","id":id,"mcpInspection":value}),Err(error)=>json!({"type":"done","id":id,"error":error})};
            let _=output.send(event).await;
        });
        Ok(Value::Null)
    }
    pub(super) fn enable_remote_mcp(
        &self,
        session: &str,
        token: &str,
        names: Vec<String>,
    ) -> Result<Value, String> {
        let mut slot = self.mcp_remote_review.lock().map_err(|_| STALE)?;
        let review = slot.as_ref().ok_or(STALE)?;
        if review.token != token
            || review.session != session
            || review.root != self.mcp_root(session)?
            || review.created.elapsed() > Duration::from_secs(300)
            || self
                .store
                .mcp_connection_by_id(&review.root, &review.connection_id)?
                != review.previous
        {
            return Err(STALE.into());
        }
        if names.is_empty()
            || names.len() > 2
            || names
                .iter()
                .enumerate()
                .any(|(i, name)| names[..i].contains(name))
        {
            return Err("Choose one or two reviewed MCP tools.".into());
        }
        let mut tools = names
            .iter()
            .map(|name| {
                review
                    .tools
                    .iter()
                    .find(|t| &t.name == name)
                    .cloned()
                    .ok_or("Choose a tool from the reviewed endpoint list.")
            })
            .collect::<Result<Vec<_>, _>>()?;
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        let prepared = if let Some(credential) = &review.credential {
            vec![(
                McpCredentialBinding {
                    name: MCP_REMOTE_BEARER.into(),
                    credential_id: uuid::Uuid::new_v4().to_string(),
                },
                credential.encoded_for(
                    Path::new(&review.root),
                    &review.connection_id,
                    &review.target,
                )?,
            )]
        } else if let Some(credential) = &review.oauth {
            vec![(
                McpCredentialBinding {
                    name: MCP_REMOTE_OAUTH.into(),
                    credential_id: uuid::Uuid::new_v4().to_string(),
                },
                credential.encoded_for(
                    Path::new(&review.root),
                    &review.connection_id,
                    &review.target,
                )?,
            )]
        } else {
            vec![]
        };
        let value = McpConnection {
            id: review.connection_id.clone(),
            revision: 1,
            enabled: true,
            launch: None,
            remote: Some(review.target.clone()),
            fingerprints: vec![],
            protocol_version: review.protocol.clone(),
            server_name: review.server_name.clone(),
            server_version: review.server_version.clone(),
            tools,
            credentials: vec![],
            retired_credentials: vec![],
        };
        let result = self.persist_mcp_review(
            &review.root,
            review.previous.as_ref(),
            review.created,
            value,
            prepared,
        )?;
        slot.take();
        Ok(result)
    }
}
