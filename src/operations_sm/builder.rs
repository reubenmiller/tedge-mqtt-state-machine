use crate::operations_sm::actor::OperationsActor;
use crate::operations_sm::config::OperationWorkflow;
use crate::operations_sm::messages::{OperationInput, OperationPluginMessage};
use log::error;
use std::convert::Infallible;
use std::process::Output;
use tedge_actors::{
    adapt, Builder, ClientMessageBox, DynSender, LoggingReceiver, Message, NoConfig,
    RuntimeRequest, RuntimeRequestSink, ServiceProvider,
};
use tedge_mqtt_ext::{MqttMessage, TopicFilter};

pub struct OperationsActorBuilder {
    input_receiver: LoggingReceiverBuilder<OperationInput>,
    mqtt_sender: DynSender<MqttMessage>,
    script_runner: ClientMessageBox<Execute, std::io::Result<Output>>,

    /// All the operation workflow definitions,
    /// possibly with a channel to the actor operation plugin that implement the workflow
    workflows: Vec<(
        TopicFilter,
        OperationWorkflow,
        Option<DynSender<OperationPluginMessage>>,
    )>,
}

impl OperationsActorBuilder {
    pub fn new(
        mqtt: &mut impl ServiceProvider<MqttMessage, MqttMessage, TopicFilter>,
        script_runner: &mut impl ServiceProvider<Execute, std::io::Result<Output>, NoConfig>,
    ) -> Self {
        let input_receiver = LoggingReceiverBuilder::new(OperationsActor::name());
        let input_sender = adapt(&input_receiver.get_input_sender());
        let mqtt_sender = mqtt.connect_consumer(OperationsActor::subscriptions(), input_sender);
        let script_runner = ClientMessageBox::new("Operation Script Runner", script_runner);
        let workflows = Vec::new();

        OperationsActorBuilder {
            input_receiver,
            mqtt_sender,
            script_runner,
            workflows,
        }
    }

    pub fn register_operation_plugin(
        &mut self,
        sender: DynSender<OperationPluginMessage>,
        workflow: OperationWorkflow,
    ) {
        let filter = &workflow.filter.clone();
        if let Err(err) = self.register_workflow(workflow, Some(sender)) {
            error!("Fail to register the plugin for {:?}: {err}", filter);
        }
    }

    pub fn register_custom_workflow(&mut self, workflow: OperationWorkflow) {
        let filter = &workflow.filter.clone();
        if let Err(err) = self.register_workflow(workflow, None) {
            error!("Fail to register the workflow for {:?}: {err}", filter);
        }
    }

    pub fn register_workflow(
        &mut self,
        workflow: OperationWorkflow,
        sender: Option<DynSender<OperationPluginMessage>>,
    ) -> Result<(), String> {
        let filter = &workflow.filter;
        let topic = filter.try_into()?;
        self.workflows.push((topic, workflow, sender));
        Ok(())
    }
}

impl ServiceProvider<OperationPluginMessage, OperationPluginMessage, OperationWorkflow>
    for OperationsActorBuilder
{
    fn connect_consumer(
        &mut self,
        config: OperationWorkflow,
        response_sender: DynSender<OperationPluginMessage>,
    ) -> DynSender<OperationPluginMessage> {
        self.register_operation_plugin(response_sender, config);
        adapt(&self.input_receiver.get_input_sender())
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
        Ok(OperationsActor::new(
            self.input_receiver.build(),
            self.mqtt_sender,
            self.script_runner,
            self.workflows,
        ))
    }
}

// ----------------
// LoggingReceiverBuilder should be move to the `tedge_actors` crate
// ----------------

use tedge_actors::futures::channel::mpsc;
use tedge_script_ext::Execute;

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
