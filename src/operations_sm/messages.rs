use crate::operations_sm::config::OperationKey;
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
