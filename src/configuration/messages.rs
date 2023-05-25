use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    /// The target of the new configuration
    target: String,

    /// The url from where the new configuration has to be downloaded
    src_url: String,

    /// The checksum to control the integrity of the configuration
    sha256: String,

    /// The path where the configuration is temporary downloaded
    tmp_path: Option<String>,
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
}
