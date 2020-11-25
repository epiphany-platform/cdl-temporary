use cache::AddressCache;
use std::sync::Arc;
use structopt::StructOpt;
use utils::metrics;
use uuid::Uuid;
use warp::Filter;

pub mod cache;
pub mod error;
pub mod handler;

#[derive(StructOpt)]
struct Config {
    #[structopt(long, env = "SCHEMA_REGISTRY_ADDR")]
    schema_registry_addr: String,
    #[structopt(long, env = "CACHE_CAPACITY")]
    cache_capacity: usize,
    #[structopt(long, env = "INPUT_PORT")]
    input_port: u16,
}

#[tokio::main]
async fn main() {
    let config = Config::from_args();

    metrics::serve();

    let address_cache = Arc::new(AddressCache::new(
        config.schema_registry_addr,
        config.cache_capacity,
    ));
    let address_filter = warp::any().map(move || address_cache.clone());
    let schema_id_filter = warp::header::header::<Uuid>("SCHEMA_ID");
    let body_filter = warp::body::aggregate();

    let single_route = warp::path!("single" / Uuid)
        .and(schema_id_filter)
        .and(address_filter.clone())
        .and_then(handler::query_single);
    let multiple_route = warp::path!("multiple" / String)
        .and(schema_id_filter)
        .and(address_filter.clone())
        .and_then(handler::query_multiple);
    let raw_route = warp::path!("raw")
        .and(body_filter)
        .and(schema_id_filter)
        .and(address_filter.clone())
        .and_then(handler::query_raw);
    let schema_route = warp::path!("schema")
        .and(schema_id_filter)
        .and(address_filter)
        .and_then(handler::query_by_schema);

    let routes = warp::get()
        .and(single_route
        .or(multiple_route)
        .or(schema_route)
        .or(raw_route));

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.input_port))
        .await;
}
