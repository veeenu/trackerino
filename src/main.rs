use log::*;
use warp::{
  Filter,
  http::Method,
};

use std::sync::{Arc, atomic::AtomicUsize};

mod logging;
mod tracking;

#[tokio::main]
async fn main() {
  logging::init();

  // TODO load the value from SQL
  // This makes the program essentially non-distributed
  // But I don't care (for now)
  let entry_id_factory = Arc::new(AtomicUsize::new(0));

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
    tracking::tracking(entry_id_factory.clone())
      .with(cors.clone())
  )
  .or(home);

  warp::serve(routes).run(([0, 0, 0, 0], 9000)).await;
}
