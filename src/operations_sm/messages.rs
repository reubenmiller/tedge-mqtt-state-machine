use crate::operations_sm::config::OperationKey;
use log::info;
use serde_json::Value;
use tedge_actors::fan_in_message_type;
use tedge_mqtt_ext::{MqttMessage, QoS};
fan_in_message_type!(OperationInput[MqttMessage, OperationPluginMessage]: Debug);

#[derive(Clone, Debug)]
pub struct OperationPluginMessage {
    pub operation: OperationKey,
    pub status: String,
    pub json: Value,
}

impl OperationPluginMessage {
    /// The status is added to the json payload
    pub fn new(operation: OperationKey, status: String, mut json: Value) -> Self {
        let json_status = status.clone();
        json.as_object_mut()
            .map(|o| o.insert("status".to_string(), json_status.into()));

        OperationPluginMessage {
            operation,
            status,
            json,
        }
    }

    /// The status is read from the json payload
    pub fn update_from_json(self, json: Value) -> Self {
        let status = json
            .as_object()
            .map(|o| o.get("status"))
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or("unknown".to_string());

        OperationPluginMessage {
            status,
            json,
            ..self
        }
    }

    pub fn update_with_script_output(
        self,
        script: String,
        output: std::io::Result<std::process::Output>,
    ) -> Self {
        match output {
            Ok(output) => {
                if output.status.success() {
                    match String::from_utf8(output.stdout) {
                        Ok(stdout) => match serde_json::from_str(&stdout) {
                            Ok(json) => {
                                info!("XOXOX: {}", &json);
                                self.update_from_json(json)
                            }
                            Err(err) => {
                                let reason =
                                    format!("Script {script} returned non JSON stdout: {err}");
                                self.failed_with(reason)
                            }
                        },
                        Err(_) => {
                            let reason = format!("Script {script} returned non UTF-8 stdout");
                            self.failed_with(reason)
                        }
                    }
                } else {
                    match String::from_utf8(output.stderr) {
                        Ok(stderr) => {
                            let reason = format!("Script {script} failed with: {stderr}");
                            self.failed_with(reason)
                        }
                        Err(_) => {
                            let reason =
                                format!("Script {script} failed and returned non UTF-8 stderr");
                            self.failed_with(reason)
                        }
                    }
                }
            }
            Err(err) => {
                let reason = format!("Failed to launch {script}: {err}");
                self.failed_with(reason)
            }
        }
    }

    pub fn failed_with(mut self, reason: String) -> Self {
        let status = "failed";
        self.json.as_object_mut().map(|o| {
            o.insert("status".to_string(), status.into());
            o.insert("reason".to_string(), reason.into());
        });

        OperationPluginMessage {
            status: status.to_owned(),
            ..self
        }
    }
}

impl TryFrom<&MqttMessage> for OperationPluginMessage {
    type Error = String;

    fn try_from(event: &MqttMessage) -> Result<Self, Self::Error> {
        let operation = OperationKey::try_from(&event.topic)?;

        let msg = event
            .payload_str()
            .map_err(|_| "Not an UTF-8 message".to_string())?;

        let json: Value =
            serde_json::from_str(msg).map_err(|_| "Not a JSON message".to_string())?;

        let status = json
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or("Missing status")?;

        Ok(OperationPluginMessage {
            operation,
            status: status.to_string(),
            json,
        })
    }
}

impl TryFrom<OperationPluginMessage> for MqttMessage {
    type Error = String;

    fn try_from(value: OperationPluginMessage) -> Result<Self, Self::Error> {
        let operation_key = &value.operation;
        let topic = operation_key.try_into()?;
        let payload = value.json.to_string();

        Ok(MqttMessage::new(&topic, payload)
            .with_qos(QoS::AtLeastOnce)
            .with_retain())
    }
}
