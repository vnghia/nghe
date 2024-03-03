mod album;
mod artist;
mod run_scan;
mod scan_full;
mod song;

pub use run_scan::run_scan;
pub use run_scan::ScanMode;

#[cfg(test)]
pub mod tests {
    pub use super::album::upsert_album;
    pub use super::artist::upsert_artists;
}
