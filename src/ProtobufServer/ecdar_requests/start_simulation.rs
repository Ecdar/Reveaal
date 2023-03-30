use crate::DataReader::component_loader::ModelCache;
use crate::ProtobufServer::services::{SimulationStartRequest, SimulationStepResponse};
use crate::ProtobufServer::ConcreteEcdarBackend;
use crate::Simulation::decision::Decision;
use crate::System::specifics::SpecificDecision;

use tonic::Status;

use super::request_util::simulation_info_to_transition_system;

impl ConcreteEcdarBackend {
    /// Handles a start simulation request: Responding with the initial decision point in the transition system given in the `request`.
    pub fn handle_start_simulation(
        request: SimulationStartRequest,
        mut cache: ModelCache,
    ) -> Result<SimulationStepResponse, Status> {
        let simulation_info = request.simulation_info.unwrap();

        let transition_system = simulation_info_to_transition_system(&simulation_info, &mut cache);

        // Get the decisions from the initial state and convert them to proto
        let initial = Decision::get_initial_decisions(&transition_system)
            .into_iter()
            .map(|i| SpecificDecision::from_decision(&i, &*transition_system).into())
            .collect();

        Ok(SimulationStepResponse {
            new_decision_points: initial,
        })
    }
}
