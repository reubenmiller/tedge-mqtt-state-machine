pub mod configuration;
pub mod operations_sm;

use crate::configuration::builder::ConfigManagerBuilder;
use crate::operations_sm::builder::OperationsActorBuilder;
use tedge_actors::Runtime;
use tedge_mqtt_ext::{MqttActorBuilder, MqttConfig};
use tedge_signal_ext::SignalActor;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mqtt_config = MqttConfig::default().with_session_name("Experimental MQTT State Machine");

    let mut runtime = Runtime::try_new(None).await?;
    let signal_actor = SignalActor::builder(&runtime.get_handle());
    let mut mqtt_actor = MqttActorBuilder::new(mqtt_config);
    let mut operations_actor = OperationsActorBuilder::new(&mut mqtt_actor);

    let config_manager = ConfigManagerBuilder::new(&mut operations_actor);

    runtime.spawn(signal_actor).await?;
    runtime.spawn(mqtt_actor).await?;
    runtime.spawn(operations_actor).await?;
    runtime.spawn(config_manager).await?;
    runtime.run_to_completion().await?;
    Ok(())
}
