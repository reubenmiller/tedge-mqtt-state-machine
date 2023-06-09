# tedge MQTT state machines

Exploring the idea of providing MQTT hooks into the thin-edge operation workflows.

## Why?

Thin-edge provides built-in support of operation workflows to manage edge devices from the cloud
and install software packages, update firmware, update configuration files ...
For each operation, thin-edge handles end-to-end all the interactions
between the cloud, the device, its child-devices and sub-systems.

This out-of-the box support is handy but too rigid.
In practice, one needs to adapt the operation workflows to specific use-cases.
A user should be able to:

- add application specific steps and checks,
- trigger the operation from a different cloud,
- target sub-systems with specific constraints.

## What?

Expose over MQTT all the different states and steps that define the workflow of an operation;
so independent sub-systems can observe the operation progress and act accordingly to their role;
the aim being to let the users adapt the operation implementation by substituing some sub-systems by their own.

## Design ideas

The core idea is to use MQTT retained messages to capture the current state of each on-going operation.

- A specific topic is attached to each operation instance.
   - `tedge/operations/${operation.target}/${operation.type}/{operation.request}/${operation.id}` 
   - e.g. `tedge/operations/main-device/configuration/update/123456`
- The messages published over this topic represent the current state of the operation workflow.
   - These messages also give the required information to proceed.
   - e.g. `{ "status": "Requested", "operation": "ConfigurationUpdate", "target": "mosquitto", "url": "https://..."}` 
- The state messages are published as retained.
   - They capture the latest state of the operation.
   - Till some change occurs, this latest state must be dispatched to any participant of the workflow on reconnect.
- Several participants act in concert to move forward the workflow.
   - The participants observe the progress of all the operations they are interested in.
   - They watch for the specific states their are responsible in moving forward.
   - When a step is performed, successfully or not, the new state is published accordingly by the performer.
- At any state for the operation, *one and only one* participant should be responsible to move the operation forward.
   - We will not try to enforce that.
   - What can be done by thin-edge?
     - Check that the observed states are known .i.e. declared in the operation configuration file.
        - Close the operation is an unknown step is observed.
     - Use the operation configuration file to tell what to do on each stage of the operation:
        - apply the default behavior provided by tedge,
        - launch a script that will perform a step and tell thin-edge what is the new state,
        - let some external participant do what is required.

## Benefits

- A ready-to-use implementation of a workflow can be adapted by a user.
  - New state can be added (as long as an external sub-system is handling this state).
  - Default behavior can be overridden (by providing an external script or delegating the step to an external participant).
- This fully decouples any cloud-specific support from the core operation workflow.
  - A cloud-specific mapper is only responsible for message translation.
  - On request from the cloud, the mapper has to create the initial state of the operation.
  - While the operation is making progress, the mapper can report this progress to the cloud end-point.
  - The mapper has to watch when a terminal state is reach to report this event accordingly and close the operation.
- An external audit system can log all the operations and their progress.

## Example

The default workflow provided by thin-edge support for configuration management is the following:

```
type = "configuration"
request = "update"

# The default behavior is to immediately schedule the new request.
# Having an init state with an automatic transition to an other step is done in order to:
# - let the users plug their own behavior to check, prepare or adapt the request,
# - while keeping unchanged the sub-systems that create these requests (i.e. the mappers).
[init]
owner = "tedge"
next = ["scheduled"]

[scheduled]
owner = "tedge"
next = ["downloading"]

[downloading]
owner = "tedge"
next = ["downloaded", "failed"]

# The default behavior is to immediately proceed with installation.
# Having a state with an automatic transition to an other step is done in order to:
# - let the users plug their own behavior to check, prepare or adapt the installation,
# - while keeping unchanged the sub-system that leads to this state (i.e. the downloader).
[downloaded]
owner = "tedge"
next = ["installing"]

[installing]
owner = "tedge"
next = ["successful", "failed"]

[successful]
owner = "tedge"
next = []

[failed]
owner = "tedge"
next = []
```

A user who wants to control when an operation update can be launched and applied can:
- use his own implementation of a service to schedule the operations,
- override the default behavior to check the download,
- add new steps, here to use a custom installation script for corner cases.

```
type = "configuration"
request = "update"

[init]
# An external daemon schedules the requests.
owner = "external"
next = ["scheduled, "failed"]"

[scheduled]
owner = "tedge"
next = ["downloading]"

[downloading]
owner = "tedge"
next = ["downloaded", "failed"]

[downloaded]
# The default behavior of thin-edge is overrided.
owner = "tedge"
script = "check.sh"
next = ["installing", "installing-custom", "failed"]

[installing]
owner = "tedge"
next = ["successful", "failed"]

[installing-custom]
# An alternative path is added.
owner = "tedge"
script = "install.sh"
next = ["successful", "failed"]

[successful]
owner = "tedge"
next = []

[failed]
owner = "tedge"
next = []
```


When the owner of a step is not `tedge` then `tedge-mqtt-state-machine` do nothing and
simply awaits that an external system (a daemon or a child device) handles the transition.

When the owner is `tedge` and a `script` is given,
this script is launched to handle the transition
and the std output of this script is used to define the new state.
This std output is expected to be in JSON and to provide at least a "status".

When the owner is `tedge` and no `script` is given,
then the step is delegated to an internal workflow.

TODO:
- [ ] Replace the fake configuration manager workflow by a real one that actually download and install the config.
- [ ] Handle the error of an internal workflow. Currently, these errors are simply logged. They must also fail the state machine.
- [ ] Use inotify to dynamically reload new user-defined workflows.


## Demo

Run the service

```shell
$ cargo build
$ RUST_LOG=debug target/debug/tedge-mqtt-state-machine
```

On start, this service loads all the workflows in `operations/*.toml`.
An example is provided: `operations/updated_configuration_operation.toml`.

On start, `tedge-mqtt-state-machine` subscribes to `tedge/operations/+/+/+/+`,
watching for workflow state updates for operations keyed as
`tedge/operations/{subsystem}/{operation}/{request}/{instance}`.

To follow what's going on, the simpler is to subscribe to the same topic:

```shell
$ tedge mqtt sub 'tedge/operations/+/+/+/+'
```

Own can then trigger an operation (mimicking the cloud mapper).

```shell
$ tedge mqtt pub \
    tedge/operations/main-device/configuration/update/123 \
    '{ "status":"init", "target":"mosquito", "src_url":"https://there", "sha256":"okay" }'
```

This event is acknowledged by `tedge-mqtt-state-machine` but nothing is done.
Indeed, in the workflow definition (`operations/updated_configuration_operation.toml`)
the owner of the `init` state is *not* `tedge`.

So one has to simulate this *external* system that is responsible for this state:

```shell
$ tedge mqtt pub \
    tedge/operations/main-device/configuration/update/123 \
    '{ "status":"scheduled", "target":"mosquito", "src_url":"https://there", "sha256":"okay" }'
```

As `tedge` is the owner of the `scheduled` state,
`tedge-mqtt-state-machine` ensures the workflow is making progress as expected.

Note that the action for the `downloaded` state is associated to the `operations/pre-install-check.sh` script.
This script has to print on stdout the new state for the workflow:
a JSON object with at least a `status` field.
This status is used to move the workflow in a new state and to determine its owner.

The workflow progress then to a final state (`successful` or `failed`).

The assumption is then that the initiator of this operation (in practice the cloud mapper)
clears the operation instance. This is done by sending a retained empty message on the associated topic.

```shell
$ tedge mqtt pub --retain tedge/operations/main-device/configuration/update/123 ''
```