use tedge_actors::fan_in_message_type;
use tedge_mqtt_ext::MqttMessage;
fan_in_message_type!(OperationInput[MqttMessage, OperationPluginResponse]: Debug);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationPluginRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationPluginResponse;
