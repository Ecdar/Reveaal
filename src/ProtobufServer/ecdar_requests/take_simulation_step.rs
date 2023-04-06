use tonic::Status;

use crate::{
    DataReader::{
        component_loader::ModelCache,
        proto_reader::{proto_decision_to_decision, simulation_info_to_transition_system},
    },
    ProtobufServer::{
        services::{SimulationStepRequest, SimulationStepResponse},
        ConcreteEcdarBackend,
    },
    System::specifics::SpecificDecision,
};

impl ConcreteEcdarBackend {
    /// Handles a take simulation step request:
    /// Given a `decision` and transition system in the `request`, walk along the decided edge and respond with the resulting decision points.
    pub fn handle_take_simulation_step(
        request: SimulationStepRequest,
        _cache: ModelCache, // TODO should be used...
    ) -> Result<SimulationStepResponse, Status> {
        let request_message = request;
        let simulation_info = request_message.simulation_info.unwrap();

        let system = simulation_info_to_transition_system(&simulation_info);

        let chosen_decision = request_message.chosen_decision.unwrap();
        let chosen_decision = proto_decision_to_decision(chosen_decision, &system);

        let decision_points = chosen_decision.resolve(&system);

        let decision_points = decision_points
            .into_iter()
            .map(|i| SpecificDecision::from_decision(&i, &*system).into())
            .collect();

        let simulation_step_response = SimulationStepResponse {
            new_decision_points: decision_points,
        };

        Ok(simulation_step_response)
    }
}
