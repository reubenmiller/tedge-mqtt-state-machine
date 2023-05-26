use crate::configuration::actor::ConfigManager;
use crate::configuration::messages::ConfigUpdateRequestState;
use crate::operations_sm::config::OperationWorkflow;
use crate::operations_sm::messages::OperationPluginMessage;
use std::convert::Infallible;
use tedge_actors::{adapt, Builder, DynSender, NoConfig, RuntimeRequest, RuntimeRequestSink, ServiceConsumer, ServiceProvider, SimpleMessageBoxBuilder};

pub struct ConfigManagerBuilder {
    message_box: SimpleMessageBoxBuilder<ConfigUpdateRequestState, ConfigUpdateRequestState>,
}

impl ConfigManagerBuilder {
    pub fn new(
        operations: &mut impl ServiceProvider<
            OperationPluginMessage,
            OperationPluginMessage,
            OperationWorkflow,
        >,
    ) -> Self {
        let message_box = SimpleMessageBoxBuilder::new(ConfigManager::name(), 16);
        let mut builder = ConfigManagerBuilder {
            message_box
        };
        builder.set_connection(operations);
        builder
    }

    pub fn workflow() -> OperationWorkflow {
        toml::from_str(include_str!("configuration_operation.toml"))
        .unwrap()
    }
}

impl ServiceConsumer<OperationPluginMessage,
    OperationPluginMessage,
    OperationWorkflow> for ConfigManagerBuilder {
    fn get_config(&self) -> OperationWorkflow {
        ConfigManagerBuilder::workflow()
    }

    fn set_request_sender(&mut self, request_sender: DynSender<OperationPluginMessage>) {
        self.message_box.set_request_sender(adapt(&request_sender))
    }

    fn get_response_sender(&self) -> DynSender<OperationPluginMessage> {
        adapt(&self.message_box.get_response_sender())
    }
}

impl ServiceProvider<ConfigUpdateRequestState, ConfigUpdateRequestState, NoConfig>
    for ConfigManagerBuilder
{
    fn connect_consumer(
        &mut self,
        config: NoConfig,
        response_sender: DynSender<ConfigUpdateRequestState>,
    ) -> DynSender<ConfigUpdateRequestState> {
        self.message_box.connect_consumer(config, response_sender)
    }
}

impl Builder<ConfigManager> for ConfigManagerBuilder {
    type Error = Infallible;

    fn try_build(self) -> Result<ConfigManager, Self::Error> {
        Ok(ConfigManager::new(self.message_box.build()))
    }
}

impl RuntimeRequestSink for ConfigManagerBuilder {
    fn get_signal_sender(&self) -> DynSender<RuntimeRequest> {
        self.message_box.get_signal_sender()
    }
}
