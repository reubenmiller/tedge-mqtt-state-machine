use std::convert::Infallible;
use tedge_actors::{
    adapt, Builder, DynSender, LoggingReceiver, Message, RuntimeRequest, RuntimeRequestSink,
    ServiceProvider,
};

use crate::operations_sm::actor::OperationsActor;
use crate::operations_sm::messages::OperationInput;
use tedge_mqtt_ext::{MqttMessage, TopicFilter};

pub struct OperationsActorBuilder {
    input_receiver: LoggingReceiverBuilder<OperationInput>,
    mqtt_sender: DynSender<MqttMessage>,
}

impl OperationsActorBuilder {
    pub fn new(mqtt: &mut impl ServiceProvider<MqttMessage, MqttMessage, TopicFilter>) -> Self {
        let input_receiver = LoggingReceiverBuilder::new(OperationsActor::name());
        let input_sender = adapt(&input_receiver.get_input_sender());
        let mqtt_sender = mqtt.connect_consumer(OperationsActor::subscriptions(), input_sender);

        OperationsActorBuilder { input_receiver, mqtt_sender }
    }
}

impl RuntimeRequestSink for OperationsActorBuilder {
    fn get_signal_sender(&self) -> DynSender<RuntimeRequest> {
        self.input_receiver.get_signal_sender()
    }
}

impl Builder<OperationsActor> for OperationsActorBuilder {
    type Error = Infallible;

    fn try_build(self) -> Result<OperationsActor, Self::Error> {
        Ok(OperationsActor::new(self.input_receiver.build(), self.mqtt_sender))
    }
}

// ----------------
// LoggingReceiverBuilder should be move to the `tedge_actors` crate
// ----------------

use tedge_actors::futures::channel::mpsc;

struct LoggingReceiverBuilder<M: Message> {
    receiver: LoggingReceiver<M>,
    input_sender: mpsc::Sender<M>,
    signal_sender: mpsc::Sender<RuntimeRequest>,
}

impl<M: Message> LoggingReceiverBuilder<M> {
    pub fn new(name: &str) -> Self {
        let (input_sender, input_receiver) = mpsc::channel(10);
        let (signal_sender, signal_receiver) = mpsc::channel(10);
        let receiver = LoggingReceiver::new(name.to_string(), input_receiver, signal_receiver);

        LoggingReceiverBuilder {
            receiver,
            input_sender,
            signal_sender,
        }
    }

    pub fn get_input_sender(&self) -> DynSender<M> {
        self.input_sender.clone().into()
    }

    pub fn build(self) -> LoggingReceiver<M> {
        self.receiver
    }
}

impl<M: Message> RuntimeRequestSink for LoggingReceiverBuilder<M> {
    fn get_signal_sender(&self) -> DynSender<RuntimeRequest> {
        self.signal_sender.clone().into()
    }
}
