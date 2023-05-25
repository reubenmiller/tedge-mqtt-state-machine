use crate::configuration::messages::{ConfigUpdateRequest, ConfigUpdateRequestState};
use async_trait::async_trait;
use std::time::Duration;
use tedge_actors::futures::stream::FuturesUnordered;
use tedge_actors::futures::StreamExt;
use tedge_actors::{Actor, MessageReceiver, RuntimeError, Sender, SimpleMessageBox};
use tokio::task::JoinHandle;

/// Using fake demo for this POC.
struct Download {
    id: String,
    request: ConfigUpdateRequest,
    path: String,
}

impl Download {
    fn new(id: &str, request: &ConfigUpdateRequest, path: &str) -> Self {
        Download {
            id: id.to_string(),
            request: request.clone(),
            path: path.to_string(),
        }
    }

    async fn execute(self) -> Self {
        tokio::time::sleep(Duration::from_secs(3)).await;
        self
    }
}

/// Actor that handles the configuration update requests.
pub struct ConfigManager {
    message_box: SimpleMessageBox<ConfigUpdateRequestState, ConfigUpdateRequestState>,
    downloads: FuturesUnordered<JoinHandle<Download>>,
}

#[async_trait]
impl Actor for ConfigManager {
    fn name(&self) -> &str {
        "ConfigManager"
    }

    async fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            let maybe_response = tokio::select! {
                Some(request) = self.message_box.recv() => {
                    match request {
                        ConfigUpdateRequestState::Init { id, request } => {
                            Some(self.init(id, request))
                        }
                        ConfigUpdateRequestState::Scheduled { id, request } => {
                            Some(self.start_download(id, request))
                        }
                        ConfigUpdateRequestState::Downloading { .. } => {
                            // This event is only useful for the other participants
                            // while this actor awaits for the actual end of the download
                            None
                        }
                        ConfigUpdateRequestState::Downloaded { id, request, path } => {
                            Some(self.start_install(id, request, path))
                        }
                        ConfigUpdateRequestState::Installing { id, request, path } => {
                            Some(self.install(id, request, path))
                        }
                        ConfigUpdateRequestState::Successful { .. } => {
                            // This event is only useful for the other participants
                            None
                        }
                        ConfigUpdateRequestState::Failed { .. } => {
                            // This event is only useful for the other participants
                            None
                        }
                    }
                }
                Some(download) = self.downloads.next() => {
                    Some(self.end_download(download.expect("fail to spawn a task")))
                }
                else => {
                    return Ok(());
                }
            };

            if let Some(response) = maybe_response {
                self.message_box.send(response).await?
            }
        }
    }
}

impl ConfigManager {
    pub fn new(
        message_box: SimpleMessageBox<ConfigUpdateRequestState, ConfigUpdateRequestState>,
    ) -> Self {
        ConfigManager {
            message_box,
            downloads: FuturesUnordered::new(),
        }
    }

    /// A new request is immediately scheduled.
    ///
    /// Having an init state with an automatic transition to an other step is done in order to:
    /// - let the users plug their own behavior to check, prepare or adapt the request,
    /// - while keeping unchanged the sub-systems that create these requests (i.e. the mappers).
    fn init(&mut self, id: String, request: ConfigUpdateRequest) -> ConfigUpdateRequestState {
        ConfigUpdateRequestState::Scheduled { id, request }
    }

    fn start_download(
        &mut self,
        id: String,
        request: ConfigUpdateRequest,
    ) -> ConfigUpdateRequestState {
        let path = format!("/tmp/configuration.download.{id}");
        let download = Download::new(&id, &request, &path);
        self.downloads.push(tokio::spawn(download.execute()));
        ConfigUpdateRequestState::Downloading { id, request, path }
    }

    fn end_download(&mut self, download: Download) -> ConfigUpdateRequestState {
        ConfigUpdateRequestState::Downloaded {
            id: download.id,
            request: download.request,
            path: download.path,
        }
    }

    /// The installation is immediately scheduled.
    ///
    /// Having a state with an automatic transition to an other step is done in order to:
    /// - let the users plug their own behavior to check, prepare or adapt the installation,
    /// - while keeping unchanged the sub-system that leads to this state (i.e. the downloader).
    fn start_install(
        &mut self,
        id: String,
        request: ConfigUpdateRequest,
        path: String,
    ) -> ConfigUpdateRequestState {
        ConfigUpdateRequestState::Installing { id, request, path }
    }

    fn install(
        &mut self,
        id: String,
        request: ConfigUpdateRequest,
        _path: String,
    ) -> ConfigUpdateRequestState {
        ConfigUpdateRequestState::Successful { id, request }
    }
}
