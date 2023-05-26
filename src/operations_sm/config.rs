use std::collections::HashMap;
use tedge_mqtt_ext::{Topic, TopicFilter};

/// An OperationKey uniquely identifies an operation instance
///
/// There is a one-to-one relationship between an OperationKey
/// and the MQTT topic on which the operation instance state are published.
///
/// `tedge/operations/{subsystem}/{operation}/{request}/{instance}`
#[derive(Clone, Debug, Default)]
pub struct OperationKey {
    /// The subsystem to which the operation applies:
    /// - the main device,
    /// - a child device,
    /// - ...
    pub subsystem: String,

    /// The operation type
    /// - configuration,
    /// - firmware update,
    /// - software update,
    /// - ...
    pub operation: String,

    /// The operation request
    /// - list
    /// - update,
    /// - remove,
    /// - ...
    pub request: String,

    /// The operation instance id
    pub instance: String,
}

impl TryFrom<&Topic> for OperationKey {
    type Error = String;

    fn try_from(topic: &Topic) -> Result<Self, Self::Error> {
        let mut subsystem = String::new();
        let mut operation = String::new();
        let mut request = String::new();
        let mut instance = String::new();
        scanf::sscanf!(
            &topic.name,
            "tedge/operations/{}/{}/{}/{}",
            subsystem,
            operation,
            request,
            instance
        )
        .map_err(|_| format!("Not an operation topic: {}", topic.name))?;
        Ok(OperationKey {
            subsystem,
            operation,
            request,
            instance,
        })
    }
}

impl TryFrom<&OperationKey> for Topic {
    type Error = String;

    fn try_from(value: &OperationKey) -> Result<Self, Self::Error> {
        let topic = format!(
            "tedge/operations/{}/{}/{}/{}",
            value.subsystem, value.operation, value.operation, value.instance,
        );
        Topic::new(&topic).map_err(|_| format!("Not a valid topic: {topic}"))
    }
}

/// An OperationFilter defines a set of operation instances.
///
/// OperationFilters are used by:
/// - Operation plugins to subscribe a specific set of operation state updates.
/// - Workflow definitions to define their scope.
///
/// An OperationFilter translates into an MQTT topic filter.
///
/// For instance, the filter of a plugin that handles all configuration related requests
/// on the main device and the child devices is `tedge/operations/+/configuration/+/+`
///
/// A workflow definition that overrides the configuration update requests on the main-device
/// is associated to the filter `tedge/operations/main-device/configuration/update/+`
#[derive(Clone, Debug)]
pub struct OperationFilter {
    /// The systems to which this filter applies
    ///
    /// None stands for any sub-system (this is the `+` MQTT wildcard).
    pub subsystem: Option<String>,

    /// The operations to which this filter applies
    ///
    /// None stands for any operation (this is the `+` MQTT wildcard).
    pub operation: Option<String>,

    /// The requests to which this filter applies
    ///
    /// None stands for any request (this is the `+` MQTT wildcard).
    pub request: Option<String>,
}

impl TryFrom<&OperationFilter> for TopicFilter {
    type Error = String;

    fn try_from(value: &OperationFilter) -> Result<Self, Self::Error> {
        let topic_filter = format!(
            "tedge/operations/{}/{}/{}/+",
            value.subsystem.as_ref().map(|s| s.as_ref()).unwrap_or("+"),
            value.operation.as_ref().map(|s| s.as_ref()).unwrap_or("+"),
            value.operation.as_ref().map(|s| s.as_ref()).unwrap_or("+")
        );
        TopicFilter::new(&topic_filter)
            .map_err(|_| format!("Not a valid topic filter: {topic_filter}"))
    }
}

/// An OperationWorkflow defines the state machine that rules an operation
pub struct OperationWorkflow {
    /// The set of operations to which these rules applies
    pub filter: OperationFilter,

    /// The states of the state machine
    pub states: HashMap<String, OperationState>,
}

/// What has to be done by thin-edge when an operation is in this state.
pub struct OperationState {
    /// The workflow participant that is responsible on moving forward the operation when in that state
    /// - tedge
    /// - external
    pub owner: String,

    /// Possibly a script to handle the operation when in that state
    pub script: Option<String>,

    /// Transitions
    pub next: Vec<String>,
}
