use log::*;
use warp::{
  Filter,
  http::Method,
};

use std::sync::{Arc, atomic::AtomicUsize};

use tokio::sync::mpsc;

mod database;
mod logging;
mod geolocation;
mod user_agent;
mod tracking;

#[tokio::main]
async fn main() {
  logging::init();

  // Let us use dotenv for configuration.
  // That way we can also choose to disable the MaxMind 
  // geolocation service if we so wish.
  dotenv::dotenv().ok();

  let geoloc = Arc::new(geolocation::Geolocation::new());

  let (ser_tx, ser_rx) = mpsc::unbounded_channel();

  // TODO load the value from SQL
  // This makes the program essentially non-distributed
  // But I don't care (for now)
  let entry_id_factory = Arc::new(AtomicUsize::new(0));

  tokio::spawn(async move {
    match database::receive_tracking_entries(ser_rx).await {
      Ok(_) => {},
      Err(e) => error!("{:?}", e)
    }
  });

  let cors = warp::cors()
    .allow_any_origin()
    .allow_methods(&[Method::GET])
    .build();

  let home = warp::path::end()
    .and(warp::header("User-Agent"))
    .map(|ua: String| {
      let s = format!("Test: {}", ua);
      info!("{}", s);
      s
    })
    .boxed();

  let routes = (
    tracking::tracking(entry_id_factory.clone(), geoloc.clone(), ser_tx.clone())
      .with(cors.clone())
  )
  .or(home);

  warp::serve(routes).run(([0, 0, 0, 0], 9000)).await;
}
