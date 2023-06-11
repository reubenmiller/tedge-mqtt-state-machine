use crate::operations_sm::config::OperationWorkflow;
use async_trait::async_trait;
use log::{error, info};
use std::process::Output;
use tedge_actors::{
    Actor, ChannelError, ClientMessageBox, DynSender, LoggingReceiver, MessageReceiver,
    RuntimeError, Sender,
};
use tedge_mqtt_ext::{MqttMessage, Topic, TopicFilter};
use tedge_script_ext::Execute;

use crate::operations_sm::messages::{OperationInput, OperationPluginMessage};

pub struct OperationsActor {
    input_receiver: LoggingReceiver<OperationInput>,
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

#[async_trait]
impl Actor for OperationsActor {
    fn name(&self) -> &str {
        OperationsActor::name()
    }

    async fn run(&mut self) -> Result<(), RuntimeError> {
        while let Some(input) = self.input_receiver.recv().await {
            match input {
                OperationInput::MqttMessage(event) => {
                    self.handle_mqtt_operation_event(event).await?
                }
                OperationInput::OperationPluginMessage(event) => {
                    self.publish_operation_plugin_event(event).await?
                }
            }
        }
        Ok(())
    }
}

impl OperationsActor {
    pub fn name() -> &'static str {
        "Operations"
    }

    pub fn subscriptions() -> TopicFilter {
        TopicFilter::new_unchecked("tedge/operations/+/+/+/+")
    }

    pub fn new(
        input_receiver: LoggingReceiver<OperationInput>,
        mqtt_sender: DynSender<MqttMessage>,
        script_runner: ClientMessageBox<Execute, std::io::Result<Output>>,
        workflows: Vec<(
            TopicFilter,
            OperationWorkflow,
            Option<DynSender<OperationPluginMessage>>,
        )>,
    ) -> Self {
        OperationsActor {
            input_receiver,
            mqtt_sender,
            script_runner,
            workflows,
        }
    }

    async fn handle_mqtt_operation_event(
        &mut self,
        event: MqttMessage,
    ) -> Result<(), ChannelError> {
        match OperationPluginMessage::try_from(&event) {
            Err(err) => {
                error!("Ignore message on {}: {err}", event.topic.name);
                Ok(())
            }
            Ok(operation_state) => self.operation_update(event.topic, operation_state).await,
        }
    }

    /// Publish over MQTT the new state for an operation
    async fn publish_operation_plugin_event(
        &mut self,
        event: OperationPluginMessage,
    ) -> Result<(), ChannelError> {
        match event.try_into() {
            Ok(mqtt_message) => {
                let mqtt_message: MqttMessage = mqtt_message;
                self.mqtt_sender.send(mqtt_message).await?
            }
            Err(err) => {
                error!("Fail to send the operation state over MQTT: {err}")
            }
        }
        Ok(())
    }

    async fn operation_update(
        &mut self,
        topic: Topic,
        operation_state: OperationPluginMessage,
    ) -> Result<(), ChannelError> {
        match self.get_workflow_state(&topic, &operation_state.status) {
            OperationAction::Done => {
                info!("Reached final state");
            }
            OperationAction::Unknown => {
                error!("Ignore operation event {}: unknown, name={}", topic.name, &operation_state.status);
            }
            OperationAction::External(external) => {
                info!(
                    "Ignore operation event {}: delegated to {external}",
                    topic.name
                );
            }
            OperationAction::Internal(mut sender) => {
                info!("Process operation event {}: builtin step", topic.name);
                sender.send(operation_state).await?
            }
            OperationAction::Script(script) => {
                info!("Process operation event {}: using {script}", topic.name);
                if let Ok(command) = Execute::try_new(format!("{} {:?}", &script, &operation_state.json.to_string()).as_str()) {
                    let output = self.script_runner.await_response(command).await?;
                    let new_state = operation_state.update_with_script_output(script, output);
                    self.publish_operation_plugin_event(new_state).await?;
                } else {
                    error!("Fail to parse the command line: {script}");
                }
            }
        }

        Ok(())
    }

    fn get_workflow_state(&self, topic: &Topic, status: &str) -> OperationAction {
        if status == "" {
            return OperationAction::Done;
        }
        for (filter, workflow, maybe_sender) in self.workflows.iter() {
            if filter.accept_topic(topic) {
                let maybe_state = workflow.states.get(status);
                if let Some(state) = maybe_state {
                    if &state.owner != "tedge" {
                        return OperationAction::External(state.owner.to_string());
                    }
                    if let Some(script) = &state.script {
                        return OperationAction::Script(script.to_string());
                    }
                    if let Some(sender) = maybe_sender {
                        return OperationAction::Internal(sender.clone());
                    }
                }
            }
        }
        OperationAction::Unknown
    }
}

pub enum OperationAction {
    Unknown,
    Done,
    External(String),
    Internal(DynSender<OperationPluginMessage>),
    Script(String),
}
