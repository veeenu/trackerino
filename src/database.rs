use crate::tracking::TrackingEntry;

use log::*;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{NO_PARAMS, params};
use tokio::sync::mpsc::UnboundedReceiver;

// Aiight, error handling
#[derive(Debug)]
pub enum ReceiverError {
  R2D2Error(r2d2::Error),
  RusqliteError(rusqlite::Error),
}

impl From<r2d2::Error> for ReceiverError {
  fn from(err: r2d2::Error) -> ReceiverError {
    ReceiverError::R2D2Error(err)
  }
}

impl From<rusqlite::Error> for ReceiverError {
  fn from(err: rusqlite::Error) -> ReceiverError {
    ReceiverError::RusqliteError(err)
  }
}

pub async fn receive_tracking_entries(mut rx: UnboundedReceiver<TrackingEntry>) 
  -> Result<(), ReceiverError>
{
  let database_path = std::env::var("TRACKERINO_SQLITE_DB")
    .unwrap_or_else(|_| String::from("trackerino.db"));
  let connection_manager = SqliteConnectionManager::file(database_path);
  let pool = r2d2::Pool::new(connection_manager)?;

  // Ugly migration
  pool.get()?
    .execute("
      CREATE TABLE IF NOT EXISTS tracking_entries (
        entry_id INTEGER NOT NULL PRIMARY KEY,
        timestamp INTEGER NOT NULL,

        ua_name TEXT NOT NULL,
        ua_platform TEXT NOT NULL,
        ua_os TEXT NOT NULL,
        ua_os_version TEXT NOT NULL,
        ua_browser_type TEXT NOT NULL,
        ua_version TEXT NOT NULL,
        ua_vendor TEXT NOT NULL,

        place_city TEXT,
        place_country TEXT,
        place_latitude REAL,
        place_longitude REAL,

        referrer TEXT,
        origin TEXT,
        path TEXT
      )
    ", NO_PARAMS)?;

  info!("Starting recv loop...");
  while let Some(entry) = rx.recv().await {
    info!("I come from MPSC: {:?}", entry);
    let pool = pool.clone();
    let conn = pool.get()?;

    let (p_city, p_country, p_latitude, p_longitude) = 
      entry.place.map(|p| (
        Some(p.city), Some(p.country), Some(p.latitude), Some(p.longitude)
      )).unwrap_or_else(|| (None, None, None, None));

    // This is super ugly, but the project's size doesn't really
    // warrant more complex stuff than this.
    let query_result = conn.execute("
      INSERT INTO tracking_entries VALUES (
        ?, ?,
        ?, ?, ?, ?, ?, ?, ?,
        ?, ?, ?, ?,
        ?, ?, ?
      )", params![
        entry.entry_id as isize, entry.timestamp,

        entry.user_agent.name,
        entry.user_agent.platform,
        entry.user_agent.os,
        entry.user_agent.os_version,
        entry.user_agent.browser_type,
        entry.user_agent.version,
        entry.user_agent.vendor,

        p_city,
        p_country,
        p_latitude,
        p_longitude,

        entry.referrer,
        entry.origin,
        entry.path
      ]);

    match query_result {
      Ok(_) => info!("Insert successful"),
      Err(e) => error!("Insert error: {:?}", e)
    }
  }

  info!("Closing recv loop...");

  Ok(())
}

