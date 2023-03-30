use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, 
    about="Reveaal is a model checking engine for ECDAR (Environment for Compositional Design and Analysis of Real Time Systems)\nFor more information about ECDAR see https://www.ecdar.net/", 
    long_about = Some("With Reveaal you can either run a single query with the 'query' command or run it as a server with the 'serve' command"))]
pub enum Args {
    /// Start a gRPC server with the protocol defined in the protobuf file
    ///
    /// Examples of usage:
    ///
    /// Reveaal serve 127.0.0.1:4242
    ///
    /// Reveaal serve -t 1 -c 50 127.0.0.1:4242
    Serve {
        /// Ip address and port to serve the gRPC server on
        #[clap(value_name = "IP:PORT")]
        endpoint: String,

        /// The number of threads to use when running queries on the server
        #[arg(short, long, default_value_t = num_cpus::get())]
        thread_count: usize,

        /// The maximal number of component saved in the server cache
        #[arg(short, long, default_value_t = 100)]
        cache_size: usize,
    },
    /// Run a query
    ///
    /// Examples of usage:
    ///
    /// Reveaal query "refinement: Researcher || Machine || Administration <= Spec" -i samples/json/EcdarUniversity
    ///
    /// Reveaal query "consistency: Machine" -i samples/json/EcdarUniversity
    ///
    /// Reveaal query "determinism: Researcher" -i samples/json/EcdarUniversity
    Query {
        /// The query to execute
        #[clap(value_name = "QUERY_TYPE: QUERY")]
        query: String,

        /// File (XML) or folder (JSON) with component definitions
        #[arg(short, long, value_name = "XML|JSON")]
        input_folder: PathBuf,

        /// Whether to disable clock reduction
        #[arg(short, long, default_value_t = false)]
        disable_clock_reduction: bool,

        /// Save file for refinement relations
        #[arg(short, long, value_name = "FILE")]
        save_refinement_relations: Option<PathBuf>,
        // TODO: Maybe add this later
        // /// The number of threads to use when running the query
        // #[arg(short, long, default_value_t = num_cpus::get())]
        // thread_count: usize,
    },
}
