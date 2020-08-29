use crate::tracking::TrackingEntry;

use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use log::*;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, NO_PARAMS};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub enum Error {
  R2D2Error(r2d2::Error),
  RusqliteError(rusqlite::Error),
}

impl From<r2d2::Error> for Error {
  fn from(err: r2d2::Error) -> Error {
    Error::R2D2Error(err)
  }
}

impl From<rusqlite::Error> for Error {
  fn from(err: rusqlite::Error) -> Error {
    Error::RusqliteError(err)
  }
}

pub async fn migrate() -> Result<(), Error> {
  let database_path =
    std::env::var("TRACKERINO_SQLITE_DB").unwrap_or_else(|_| String::from("trackerino.db"));
  let connection_manager = SqliteConnectionManager::file(database_path);
  let pool = r2d2::Pool::new(connection_manager)?;

  // Ugly migration
  pool.get()?.execute(
    "
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
    ",
    NO_PARAMS,
  )?;

  Ok(())
}

pub async fn get_last_entry_id() -> Result<Option<usize>, Error> {
  let database_path =
    std::env::var("TRACKERINO_SQLITE_DB").unwrap_or_else(|_| String::from("trackerino.db"));
  let connection_manager = SqliteConnectionManager::file(database_path);
  let pool = r2d2::Pool::new(connection_manager)?;

  pool
    .get()?
    .query_row(
      "SELECT MAX(entry_id) FROM tracking_entries",
      NO_PARAMS,
      |row| row.get(0),
    )
    .map(|val: Option<isize>| val.map(|val| val as usize))
    .map_err(|e| e.into())
}

fn insert_entry(entry: TrackingEntry, stmt: &mut rusqlite::Statement) {
  let (p_city, p_country, p_latitude, p_longitude) = entry
    .place
    .map(|p| (p.city, p.country, p.latitude, p.longitude))
    .unwrap_or_else(|| (None, None, None, None));

  let query_result = stmt.execute(params![
    entry.entry_id as isize,
    entry.timestamp,
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
    Ok(_) => info!("Insert successful ({})", entry.entry_id),
    Err(e) => error!("Insert error: {:?}", e),
  }
}

fn insert_entries(
  conn: r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>,
  rx: mpsc::Receiver<TrackingEntry>,
) -> Result<(), Error> {
  let mut stmt = conn.prepare(
    "INSERT INTO tracking_entries VALUES (
      ?, ?,
      ?, ?, ?, ?, ?, ?, ?,
      ?, ?, ?, ?,
      ?, ?, ?
    )",
  )?;

  let mut buffer: Vec<TrackingEntry> = Vec::new();
  let mut last_flush = Instant::now();

  //while let Ok(entry) = rx.try_recv() {
  loop {
    match rx.recv_timeout(Duration::from_millis(100)) {
      Err(RecvTimeoutError::Timeout) => {}
      Err(RecvTimeoutError::Disconnected) => break,
      Ok(entry) => {
        buffer.push(entry);
      }
    }

    if buffer.len() > 1000 || last_flush.elapsed() > Duration::from_secs(5) {
      conn.execute("BEGIN TRANSACTION", NO_PARAMS).ok();
      buffer
        .into_iter()
        .for_each(|entry| insert_entry(entry, &mut stmt));
      buffer = Vec::new();
      last_flush = Instant::now();
      conn.execute("COMMIT", NO_PARAMS).ok();
    }
  }

  Ok(())
}

pub async fn receive_tracking_entries(
  mut rx: UnboundedReceiver<TrackingEntry>,
) -> Result<(), Error> {
  let database_path =
    std::env::var("TRACKERINO_SQLITE_DB").unwrap_or_else(|_| String::from("trackerino.db"));
  let connection_manager = SqliteConnectionManager::file(database_path);
  let pool = r2d2::Pool::new(connection_manager)?;

  let conn = pool.get()?;
  let (sql_tx, sql_rx) = mpsc::channel::<TrackingEntry>();

  let insert_thread = std::thread::spawn(move || insert_entries(conn, sql_rx).ok());

  info!("Starting recv loop...");
  while let Some(entry) = rx.recv().await {
    // info!("I come from MPSC: {:?}", entry);
    //let pool = pool.clone();
    let sql_tx = sql_tx.clone();

    sql_tx.send(entry).ok();

    // This is super ugly, but the project's size doesn't really
    // warrant more complex stuff than this.
  }

  insert_thread.join().ok();

  info!("Closing recv loop...");

  Ok(())
}
