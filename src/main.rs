use tedge_actors::Runtime;
use tedge_mqtt_ext::{MqttActorBuilder, MqttConfig};
use tedge_signal_ext::SignalActor;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mqtt_config = MqttConfig::default().with_session_name("Experimental MQTT State Machine");

    let mut runtime = Runtime::try_new(None).await?;
    let signal_actor = SignalActor::builder(&runtime.get_handle());
    let mqtt_actor = MqttActorBuilder::new(mqtt_config);

    runtime.spawn(signal_actor).await?;
    runtime.spawn(mqtt_actor).await?;
    runtime.run_to_completion().await?;
    Ok(())
}
