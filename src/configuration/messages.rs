use serde::Deserialize;
use serde::Serialize;
use crate::operations_sm::messages::OperationPluginMessage;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    /// The target of the new configuration
    target: String,

    /// The url from where the new configuration has to be downloaded
    src_url: String,

    /// The checksum to control the integrity of the configuration
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConfigUpdateRequestState {
    Init {
        id: String,
        request: ConfigUpdateRequest,
    },
    Scheduled {
        id: String,
        request: ConfigUpdateRequest,
    },
    Downloading {
        id: String,
        request: ConfigUpdateRequest,
        path: String,
    },
    Downloaded {
        id: String,
        request: ConfigUpdateRequest,
        path: String,
    },
    Installing {
        id: String,
        request: ConfigUpdateRequest,
        path: String,
    },
    Successful {
        id: String,
        request: ConfigUpdateRequest,
    },
    Failed {
        id: String,
        request: ConfigUpdateRequest,
        reason: String,
    },
    InvalidState {
        id: String,
        error: String,
    }
}

impl From<OperationPluginMessage> for ConfigUpdateRequestState {
    fn from(message: OperationPluginMessage) -> Self {
        let key = &message.operation;
        let id: String = key.into();
        let path = message.json.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let reason = message.json.get("error").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let request = match serde_json::from_value(message.json) {
            Ok(request) => request,
            Err(err) => {
                return ConfigUpdateRequestState::InvalidState {
                    id,
                    error: format!("Invalid configuration update state: {err}"),
                }
            }
        };
        match message.status.as_str() {
            "init" => {
                ConfigUpdateRequestState::Init {
                    id,
                    request,
                }
            }
            "scheduled" => {
                ConfigUpdateRequestState::Scheduled {
                    id,
                    request,
                }
            }
            "downloading" => {
                ConfigUpdateRequestState::Downloading {
                    id,
                    request,
                    path,
                }
            }
            "downloaded" => {
                ConfigUpdateRequestState::Downloaded {
                    id,
                    request,
                    path,
                }
            }
            "installing" => {
                ConfigUpdateRequestState::Installing {
                    id,
                    request,
                    path,
                }
            }
            "successful" => {
                ConfigUpdateRequestState::Successful {
                    id,
                    request,
                }
            }
            "failed" => {
                ConfigUpdateRequestState::Failed {
                    id,
                    request,
                    reason
                }
            }
            unknown => {
                return ConfigUpdateRequestState::InvalidState {
                    id,
                    error: format!("Invalid configuration update state: unknown status: {unknown}"),
                }
            }
        }
    }
}

impl From<ConfigUpdateRequestState> for OperationPluginMessage {
    fn from(state: ConfigUpdateRequestState) -> Self {
        let operation = state.id().try_into().expect("A valid topic");
        let status = state.status().to_string();
        let path = state.path();
        let json = match state.request() {
            Some(request) => {
                let mut json = serde_json::to_value(request).unwrap();
                if let Some(path) = path {
                    json.as_object_mut().map(|o| o.insert("path".to_string(), path.into()));
                };
                json
            }
            None => {
                serde_json::to_value(&format!(r#"{{ "status":"invalid" }}"#)).unwrap()
            }
        };

        OperationPluginMessage {
            operation,
            status,
            json
        }
    }
}

impl ConfigUpdateRequestState {
    pub fn status(&self) -> &'static str {
        match self {
            ConfigUpdateRequestState::Init { .. } => { "init" }
            ConfigUpdateRequestState::Scheduled { .. } => { "scheduled" }
            ConfigUpdateRequestState::Downloading { .. } => { "downloading" }
            ConfigUpdateRequestState::Downloaded { .. } => { "downloaded" }
            ConfigUpdateRequestState::Installing { .. } => { "installing" }
            ConfigUpdateRequestState::Successful { .. } => { "successful" }
            ConfigUpdateRequestState::Failed { .. } => { "failed" }
            ConfigUpdateRequestState::InvalidState { .. } => { "invalid" }
        }
    }
    pub fn request(&self) -> Option<&ConfigUpdateRequest> {
        match self {
            ConfigUpdateRequestState::Init { request, .. } |
            ConfigUpdateRequestState::Scheduled { request, .. } |
            ConfigUpdateRequestState::Downloading { request, .. } |
            ConfigUpdateRequestState::Downloaded { request, .. } |
            ConfigUpdateRequestState::Installing { request, .. } |
            ConfigUpdateRequestState::Successful { request, .. } |
            ConfigUpdateRequestState::Failed { request, .. } => {
                Some(request)
            }
            ConfigUpdateRequestState::InvalidState { .. } => {
                None
            }
        }
    }
    pub fn path(&self) -> Option<String> {
        match self {
            ConfigUpdateRequestState::Downloading { path, .. } |
            ConfigUpdateRequestState::Downloaded { path, .. } |
            ConfigUpdateRequestState::Installing { path, .. } => {
                Some(path.to_string())
            }
            ConfigUpdateRequestState::Init { .. } |
            ConfigUpdateRequestState::Scheduled { .. } |
            ConfigUpdateRequestState::Successful { .. } |
            ConfigUpdateRequestState::Failed { .. } |
            ConfigUpdateRequestState::InvalidState { .. } => {
                None
            }
        }
    }
    pub fn id(&self) -> &String {
        match self {
            ConfigUpdateRequestState::Init { id, .. } |
            ConfigUpdateRequestState::Scheduled { id, .. } |
            ConfigUpdateRequestState::Downloading { id, .. } |
            ConfigUpdateRequestState::Downloaded { id, .. } |
            ConfigUpdateRequestState::Installing { id, .. } |
            ConfigUpdateRequestState::Successful { id, .. } |
            ConfigUpdateRequestState::Failed { id, .. } |
            ConfigUpdateRequestState::InvalidState { id, .. } => {
                id
            }
        }
    }
}