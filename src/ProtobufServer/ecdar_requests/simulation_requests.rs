use tonic::Status;

use crate::{
    DataReader::{
        component_loader::ModelCache,
        proto_reader::{proto_decision_to_decision, simulation_info_to_transition_system},
    },
    ProtobufServer::{
        services::{SimulationStartRequest, SimulationStepRequest, SimulationStepResponse},
        ConcreteEcdarBackend,
    },
    System::specifics::SpecificDecision,
};

use crate::Simulation::decision::Decision;

impl ConcreteEcdarBackend {
    /// Handles a start simulation request: Responding with the initial decision point in the transition system given in the `request`.
    pub fn handle_start_simulation(
        request: SimulationStartRequest,
        _cache: ModelCache, // TODO should be used...
    ) -> Result<SimulationStepResponse, Status> {
        let simulation_info = request.simulation_info.unwrap();

        let transition_system = simulation_info_to_transition_system(&simulation_info);

        // Get the decisions from the initial state and convert them to proto
        let initial = Decision::get_initial_decisions(&transition_system)
            .into_iter()
            .map(|i| SpecificDecision::from_decision(&i, &*transition_system).into())
            .collect();

        Ok(SimulationStepResponse {
            new_decision_points: initial,
        })
    }

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
