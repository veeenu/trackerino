use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering} };

use crate::geolocation::{Geolocation, Place};
use crate::user_agent::UserAgent;

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
  timestamp: DateTime<Utc>,
  user_agent: UserAgent,
  referrer: Option<String>,
  origin: Option<String>,
  path: Option<String>,
  place: Option<Place>,
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

pub fn tracking(entry_id: Arc<AtomicUsize>, geoloc: Arc<Geolocation>) -> BoxedFilter<(impl Reply,)> {
  let geoloc = geoloc.clone();

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
      
      let place = addr.map(|addr| match addr {
        SocketAddr::V4(i) => geoloc.lookup(IpAddr::V4(i.ip().to_owned())),
        SocketAddr::V6(i) => geoloc.lookup(IpAddr::V6(i.ip().to_owned())),
      }).unwrap_or(None);

      let ua = ua.map(|ua| UserAgent::from(ua.as_str())).unwrap_or(UserAgent::default());

      info!("IP: {:?}", ip);
      info!("PL: {:?}", place);
      info!("UA: {:?}", ua);
      info!("{:?}", tq);
      
      let entry = TrackingEntry {
        entry_id: entry_id.fetch_add(1, Ordering::Relaxed),
        timestamp: Utc::now(),
        user_agent: ua,
        referrer: tq.referrer,
        origin: tq.origin,
        path: tq.path,
        place
      };

      format!("{:?}", entry)
    })
    .boxed()
}

// That's ok for me. I now want to build a coalesced row data structure which I can
// shuttle over to another thread to be serialized into SQLite
