use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use crate::geolocation::{Geolocation, Place};
use crate::user_agent::UserAgent;

use chrono::prelude::*;
use log::*;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::UnboundedSender;
use warp::{
  filters::BoxedFilter,
  Filter,
  Reply,
};

// This struct will be used elsewhere. I think I don't really care about
// making all the fields public, as it'll be enough to have `Serialize`
// and `Deserialize` implemented for the database to work in a transparent
// way. Of course I'll have to test this assumption.
#[derive(Deserialize, Serialize, Debug)]
pub struct TrackingEntry {
  // This ID will be loaded out of an AtomicUsize
  pub entry_id: usize,
  pub timestamp: DateTime<Utc>,
  pub user_agent: UserAgent,
  pub referrer: Option<String>,
  pub origin: Option<String>,
  pub path: Option<String>,
  pub place: Option<Place>,
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

// Ok, we need to add one parameter to this function.
// That is, I want to shuttle the Entry resulting from accessing the tracking endpoint
// to a separate thread which will handle database serialization. The way this will go
// down is as follows: I'll create the thread responsible for this memery in a separate
// module, then use std::sync::mpsc for communication. The serializer thread will get
// the receiver, the tracking filter will get the sender. This way, tokio can do
// whatever it wants with its concurrency, be it single threaded or multithreaded, and
// the receiver will be a synchronous thingy with its queue which will not affect the
// web service performance-wise. I foresee backpressure problems under very high loads,
// that is, if the serialization is significantly slower than the number of requests
// served, the memory will fill up in the queue and won't be discarded until the 
// serializer thread has used it. But that's a problem for extreme scale. Let's not
// get ahead of ourselves.

pub fn tracking(entry_id: Arc<AtomicUsize>, geoloc: Arc<Geolocation>, tx: UnboundedSender<TrackingEntry>) -> BoxedFilter<(impl Reply,)> {
  let geoloc = geoloc.clone();

  warp::path("tracking")
    .and(warp::path::end())
    .and(warp::addr::remote())
    .and(warp::filters::header::optional("User-Agent"))
    .and(warp::query::<TrackingQuery>())
    .map(move |addr: Option<SocketAddr>, ua: Option<String>, tq: TrackingQuery| {
      /*let ip = addr.map(|addr| match addr {
        SocketAddr::V4(i) => i.ip().to_string(),
        SocketAddr::V6(i) => i.ip().to_string(),
      });*/
      
      let place = addr.map(|addr| match addr {
        SocketAddr::V4(i) => geoloc.lookup(IpAddr::V4(i.ip().to_owned())),
        SocketAddr::V6(i) => geoloc.lookup(IpAddr::V6(i.ip().to_owned())),
      }).unwrap_or(None);

      let ua = ua.map(|ua| UserAgent::from(ua.as_str())).unwrap_or(UserAgent::default());

      let entry = TrackingEntry {
        entry_id: entry_id.fetch_add(1, Ordering::Relaxed),
        timestamp: Utc::now(),
        user_agent: ua,
        referrer: tq.referrer,
        origin: tq.origin,
        path: tq.path,
        place
      };

      if let Err(e) = tx.clone().send(entry) {
        error!("Threading serialization error: {:?}", e);
      }

      // This should return something otherwise the client will only get a 200
      // which would be totally fine as we don't need deep introspection tbh
      // but still. Let's wait and see if that'll ever be a valid use case.
      String::new()
    })
    .boxed()
}

