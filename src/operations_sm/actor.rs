use async_trait::async_trait;
use tedge_actors::{Actor, DynSender, LoggingReceiver, RuntimeError};
use tedge_mqtt_ext::{MqttMessage, TopicFilter};

use crate::operations_sm::messages::OperationInput;

pub struct OperationsActor {
    input_receiver: LoggingReceiver<OperationInput>,
    mqtt_sender: DynSender<MqttMessage>,
}

#[async_trait]
impl Actor for OperationsActor {
    fn name(&self) -> &str {
        OperationsActor::name()
    }

    async fn run(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

impl OperationsActor {
    pub fn name() -> &'static str {
        "Operations"
    }

    pub fn subscriptions() -> TopicFilter {
        TopicFilter::new_unchecked("tedge/operations/+/+/+")
    }

    pub fn new(
        input_receiver: LoggingReceiver<OperationInput>,
        mqtt_sender: DynSender<MqttMessage>,
    ) -> Self {
        OperationsActor { input_receiver, mqtt_sender }
    }
}
