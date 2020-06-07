use std::net::SocketAddr;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering} };

use chrono::prelude::*;
use log::*;
use serde::{Serialize, Deserialize};
use warp::{
  filters::BoxedFilter,
  Filter,
  Reply,
};

#[derive(Serialize, Debug)]
pub struct TrackingEntry {
  // This ID will be loaded out of an AtomicUsize
  entry_id: usize,
  // Would it be too aggressive if I implemented a "time spent on page" feature?
  // Like with a second field which gets  updated by ID at page-change. Would be
  // a lot of client-side event handlers. I'll keep it for later.
  timestamp: DateTime<Utc>,
  end_timestamp: Option<String>,
  ip: Option<String>,
  user_agent: Option<String>,
  referrer: Option<String>,
  origin: Option<String>,
  path: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TrackingQuery {
  // Looks like I won't care about other stuff from the client side.
  #[serde(rename = "ref")]
  referrer: Option<String>,   // document.referrer
  #[serde(rename = "ori")]
  origin: Option<String>,     // document.location.origin
  path: Option<String>,       // document.location.pathname
}

pub fn tracking(entry_id: Arc<AtomicUsize>) -> BoxedFilter<(impl Reply,)> {
  warp::path("tracking")
    .and(warp::path::end())
    .and(warp::addr::remote())
    .and(warp::filters::header::optional("User-Agent"))
    .and(warp::query::<TrackingQuery>())
    .map(move |addr: Option<SocketAddr>, ua: Option<String>, tq: TrackingQuery| {
      let ip = addr.map(|addr| match addr {
        SocketAddr::V4(i) => i.ip().to_string(),
        SocketAddr::V6(i) => i.ip().to_string(),
      });
      info!("IP: {:?}", ip);
      info!("UA: {:?}", ua);
      info!("{:?}", tq);
      
      let entry = TrackingEntry {
        entry_id: entry_id.fetch_add(1, Ordering::Relaxed),
        timestamp: Utc::now(),
        end_timestamp: None,
        ip,
        user_agent: ua,
        referrer: tq.referrer,
        origin: tq.origin,
        path: tq.path,
      };

      format!("{:?}", entry)
    })
    .boxed()
}

// That's ok for me. I now want to build a coalesced row data structure which I can
// shuttle over to another thread to be serialized into SQLite
