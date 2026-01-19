//! SNOMED CT gRPC Server binary.

use snomed_loader::{discover_rf2_files, SnomedStore};
use snomed_service::proto::{
    concept_service_server::ConceptServiceServer,
    ecl_service_server::EclServiceServer,
    search_service_server::SearchServiceServer,
};
use snomed_service::SnomedServer;
use tonic::transport::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_PORT: u16 = 50051;
const DEFAULT_DATA_PATH: &str = "H:/3.0/apps/snomed-ct-loader-rust/data/SnomedCT_InternationalRF2_PRODUCTION_20251201T120000Z";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Get data path from env or use default
    let data_path = std::env::var("SNOMED_DATA_PATH")
        .unwrap_or_else(|_| DEFAULT_DATA_PATH.to_string());

    tracing::info!("Loading SNOMED CT data from: {}", data_path);

    // Discover RF2 files
    let files = discover_rf2_files(&data_path)?;
    tracing::info!("Discovered RF2 files: {:?}", files.release_date);

    // Load data into store
    let mut store = SnomedStore::new();

    tracing::info!("Loading concepts...");
    store.load_all(&files)?;

    tracing::info!(
        "Loaded {} concepts, {} descriptions, {} relationships",
        store.concept_count(),
        store.description_count(),
        store.relationship_count()
    );

    // Create server
    let server = SnomedServer::new(store);

    // Get port from env or use default
    let port = std::env::var("SNOMED_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let addr = format!("[::1]:{}", port).parse()?;
    tracing::info!("Starting SNOMED CT gRPC server on {}", addr);

    // Start gRPC server with all services
    tracing::info!("Services available: ConceptService, SearchService, EclService");

    Server::builder()
        .add_service(ConceptServiceServer::new(server.clone()))
        .add_service(SearchServiceServer::new(server.clone()))
        .add_service(EclServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
