//! SNOMED CT gRPC Server binary.

use snomed_loader::{discover_rf2_files, SnomedStore};
use snomed_service::proto::{
    concept_service_server::ConceptServiceServer,
    ecl_service_server::EclServiceServer,
    refset_service_server::RefsetServiceServer,
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

    tracing::info!("Loading concepts, descriptions, and relationships...");
    store.load_all(&files)?;

    tracing::info!(
        "Loaded {} concepts, {} descriptions, {} relationships",
        store.concept_count(),
        store.description_count(),
        store.relationship_count()
    );

    // Load MRCM constraints if available
    tracing::info!("Loading MRCM constraints...");
    if let Err(e) = store.load_mrcm(&files) {
        tracing::warn!("Could not load MRCM data: {}", e);
    } else if store.has_mrcm() {
        tracing::info!("MRCM constraints loaded successfully");
    }

    // Load reference sets if available
    tracing::info!("Loading reference sets...");
    match store.load_simple_refsets(&files, snomed_loader::Rf2Config::default()) {
        Ok(count) => tracing::info!("Loaded {} refset members across {} refsets", count, store.refset_count()),
        Err(e) => tracing::warn!("Could not load refsets: {}", e),
    }

    // Load OWL expressions if available
    tracing::info!("Loading OWL expressions...");
    match store.load_owl_expressions(&files, snomed_loader::Rf2Config::default()) {
        Ok(count) => tracing::info!("Loaded {} OWL expressions", count),
        Err(e) => tracing::warn!("Could not load OWL expressions: {}", e),
    }

    // Load concrete relationships if available
    tracing::info!("Loading concrete relationships...");
    match store.load_concrete_relationships(&files, snomed_loader::Rf2Config::default()) {
        Ok(count) => tracing::info!("Loaded {} concrete relationships", count),
        Err(e) => tracing::warn!("Could not load concrete relationships: {}", e),
    }

    // Load language refsets if available
    tracing::info!("Loading language refsets...");
    match store.load_language_refsets(&files, snomed_loader::Rf2Config::default()) {
        Ok(count) => tracing::info!("Loaded {} language refset members", count),
        Err(e) => tracing::warn!("Could not load language refsets: {}", e),
    }

    // Load association refsets if available
    tracing::info!("Loading association refsets...");
    match store.load_association_refsets(&files, snomed_loader::Rf2Config::default()) {
        Ok(count) => tracing::info!("Loaded {} association refset members", count),
        Err(e) => tracing::warn!("Could not load association refsets: {}", e),
    }

    // Build transitive closure for O(1) hierarchy queries
    tracing::info!("Building transitive closure for optimized hierarchy queries...");
    store.build_transitive_closure();
    tracing::info!("Transitive closure built - hierarchy queries now O(1)");

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
    tracing::info!("Services available: ConceptService, SearchService, EclService, RefsetService");

    Server::builder()
        .add_service(ConceptServiceServer::new(server.clone()))
        .add_service(SearchServiceServer::new(server.clone()))
        .add_service(EclServiceServer::new(server.clone()))
        .add_service(RefsetServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
