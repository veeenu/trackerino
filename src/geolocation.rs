// This module will have a single entry point: a function
// which takes an IP and returns an Option<T> where T is
// whatever I decide for the geolocation information to be.
// Could be a String for the location, a struct with all
// the data, lat+lon or something else. Let's see what the
// API offers.

use std::net::IpAddr;
use std::str::FromStr;
use std::env;

use log::*;
use maxminddb::geoip2;
use serde::{Serialize, Deserialize};

pub struct Geolocation {
  reader: Option<maxminddb::Reader<Vec<u8>>>,
}

// I want a newtype to extract only information I have any use of. Like City and
// Country should  be more  than enough  for decent  tracking. This  should only
// serve non-commercial  purposes in my  mind so it's  useless to have  info for
// super in depth analyses. I'll only get City, Country, Latitude and Longitude.

#[derive(Debug, Serialize, Deserialize)]
pub struct Place {
  city: Option<String>,
  country: Option<String>,
  latitude: Option<f64>,
  longitude: Option<f64>,
}

// Let's trim the incoming data structure in order to extract a much leaner data
// structure which we can have any use at all for
impl From<geoip2::City<'_>> for Place {
  fn from(source: geoip2::City) -> Place {
    let (latitude, longitude) = source.location
      .map(|location| {
        (location.latitude, location.longitude)
      })
      .unwrap_or((None, None));
    let city: Option<String> = source.city
      .map(|city| city.names)
      .map(|names| names.map(|n| n.get("en").cloned()).unwrap_or(None))
      .unwrap_or(None)
      .map(|s| s.to_owned());
    let country: Option<String> = source.country
      .map(|country| country.names)
      .map(|names| names.map(|n| n.get("en").cloned()).unwrap_or(None))
      .unwrap_or(None)
      .map(|s| s.to_owned());

    Place {
      city, country, latitude, longitude
    }
  }
}

impl Geolocation {
  // Return a new instance of Geolocation, configured via environment variables.
  pub fn new() -> Geolocation {
    let service_enabled: bool = env::var("TRACKERINO_GEOLOC_USE")
      .map(|v| v == "true")
      .unwrap_or(false);

    // Let us use a sane default.
    let geolite_file: String = env::var("TRACKERINO_GEOLOC_DB")
      .unwrap_or_else(|_| "data/GeoLite2-City.mmdb".to_owned());

    if service_enabled {
      Geolocation {
        reader: match maxminddb::Reader::open_readfile(&geolite_file) {
          Ok(reader) => Some(reader),
          Err(e) => {
            error!("Could not load GeoLite2 file {}: {:?}. Geolocation disabled",
              geolite_file,
              e);
            None
          }
        }
      }
    } else {
      info!("Geolocation service disabled by settings");
      Geolocation {
        reader: None
      }
    }
  }

  pub fn lookup(&self, ip: IpAddr) -> Option<Place> {
    self.reader.as_ref().and_then(|r| {
      match r.lookup::<geoip2::City>(ip) {
        Ok(city) => Some(Place::from(city)),
        Err(e) => {
          error!("MaxMindDB lookup error: {:?}", e);
          None
        }
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use maxminddb::geoip2;
  use std::net::IpAddr;
  use std::str::FromStr;
  use std::env;

  use super::*;

  // In order to run tests:
  // - Pass --test-threads=1 because of env variables usage.
  // - Download a database from MaxMindDB to the data/GeoLite2-City.mmdb file.
  // All tests are ignored by default.
  // Wonder if there is a way to conditionally ignore tests?
  // Not really important now, will look it up later.

  #[test]
  fn test_legit_city() {
    // Users will have to download their  own GeoLite2.mmdb file as it is behind
    // an account, despite being free.
    env::set_var("TRACKERINO_GEOLOC_USE", "true");
    env::set_var("TRACKERINO_GEOLOC_DB", "data/GeoLite2-City.mmdb");

    let geoloc = Geolocation::new();

    let ip: IpAddr = FromStr::from_str("79.24.176.207")
      .unwrap();
    let city: Place= geoloc.lookup(ip).unwrap();
    println!("{:?}", city);
  }

  #[test]
  #[ignore]
  fn test_nonexistent_file() {
    env::set_var("TRACKERINO_GEOLOC_USE", "true");
    env::set_var("TRACKERINO_GEOLOC_DB", "data/i_dont_exist.mmdb");

    let geoloc = Geolocation::new();
    let ip: IpAddr = FromStr::from_str("79.24.176.207")
      .unwrap();
    let city: Option<Place> = geoloc.lookup(ip);
    println!("{:?}", city);
    assert!(city.is_none());
  }

  #[test]
  #[ignore]
  fn test_wacky_ip() {
    env::set_var("TRACKERINO_GEOLOC_USE", "true");
    env::set_var("TRACKERINO_GEOLOC_DB", "data/GeoLite2-City.mmdb");

    let geoloc = Geolocation::new();

    let ip: IpAddr = FromStr::from_str("255.255.255.255")
      .unwrap();
    let city: Option<Place> = geoloc.lookup(ip);
    println!("{:?}", city);
    assert!(city.is_none());
  }
}

