use crate::configuration::actor::ConfigManager;
use crate::configuration::messages::ConfigUpdateRequestState;
use std::convert::Infallible;
use tedge_actors::{
    Builder, DynSender, NoConfig, RuntimeRequest, RuntimeRequestSink, ServiceProvider,
    SimpleMessageBoxBuilder,
};

pub struct ConfigManagerBuilder {
    message_box: SimpleMessageBoxBuilder<ConfigUpdateRequestState, ConfigUpdateRequestState>,
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
