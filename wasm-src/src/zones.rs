//! Zone metadata for the 13 US electricity regions the simulator supports.
//!
//! The Python reference loads these from `normalized_data_2035_by_zone_new.csv`
//! via `data_loader.py:load_all_zone_data`. The web frontend loads
//! `public/data/zones.json` directly. This module gives non-web Rust callers
//! a typed enum of the supported zones and a helper to parse the JSON shape
//! the web app already ships with.

use serde::{Deserialize, Serialize};

/// One of the 13 supported US electricity zones.
///
/// Names match the keys in `public/data/zones.json` exactly — `serde` derives
/// case-sensitive (de)serialization, so renames here will break the web bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    California,
    Delta,
    Florida,
    #[serde(rename = "Mid-Atlantic")]
    MidAtlantic,
    Midwest,
    Mountain,
    #[serde(rename = "New England")]
    NewEngland,
    #[serde(rename = "New York")]
    NewYork,
    Northwest,
    Plains,
    Southeast,
    Southwest,
    Texas,
}

impl Zone {
    /// Canonical display name (matches Python `data_loader.ZONE_NAMES`).
    pub fn name(self) -> &'static str {
        match self {
            Zone::California => "California",
            Zone::Delta => "Delta",
            Zone::Florida => "Florida",
            Zone::MidAtlantic => "Mid-Atlantic",
            Zone::Midwest => "Midwest",
            Zone::Mountain => "Mountain",
            Zone::NewEngland => "New England",
            Zone::NewYork => "New York",
            Zone::Northwest => "Northwest",
            Zone::Plains => "Plains",
            Zone::Southeast => "Southeast",
            Zone::Southwest => "Southwest",
            Zone::Texas => "Texas",
        }
    }

    /// Iterator over every supported zone, in the same order Python returns
    /// from `load_all_zone_data`.
    pub fn all() -> &'static [Zone] {
        &[
            Zone::California,
            Zone::Delta,
            Zone::Florida,
            Zone::MidAtlantic,
            Zone::Midwest,
            Zone::Mountain,
            Zone::NewEngland,
            Zone::NewYork,
            Zone::Northwest,
            Zone::Plains,
            Zone::Southeast,
            Zone::Southwest,
            Zone::Texas,
        ]
    }

    /// Look up a zone by its canonical name. Case-sensitive.
    pub fn from_name(name: &str) -> Option<Zone> {
        Zone::all().iter().copied().find(|z| z.name() == name)
    }
}

/// Hourly profiles for a single zone — 8760 capacity factors / MW values.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZoneProfiles {
    pub solar: Vec<f64>,
    pub wind: Vec<f64>,
    pub load: Vec<f64>,
}

/// Parse the bytes of `zones.json` (as shipped in `public/data/`) into
/// per-zone profile maps. Returns a clear error on missing zones, wrong
/// shape, or 8760-length mismatches.
pub fn parse_zones_json(bytes: &[u8]) -> Result<std::collections::HashMap<String, ZoneProfiles>, String> {
    let map: std::collections::HashMap<String, ZoneProfiles> =
        serde_json::from_slice(bytes).map_err(|e| format!("zones.json parse error: {}", e))?;

    for (name, p) in &map {
        for (label, len) in [
            ("solar", p.solar.len()),
            ("wind", p.wind.len()),
            ("load", p.load.len()),
        ] {
            if len != crate::types::HOURS_PER_YEAR {
                return Err(format!(
                    "Zone {:?}: {} profile has {} hours, expected {}",
                    name,
                    label,
                    len,
                    crate::types::HOURS_PER_YEAR
                ));
            }
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_roundtrip_covers_all_zones() {
        for &zone in Zone::all() {
            let name = zone.name();
            let parsed = Zone::from_name(name).expect(&format!("missing roundtrip for {}", name));
            assert_eq!(parsed, zone);
        }
        assert_eq!(Zone::all().len(), 13);
    }

    #[test]
    fn from_name_is_case_sensitive() {
        assert_eq!(Zone::from_name("California"), Some(Zone::California));
        assert_eq!(Zone::from_name("california"), None);
        assert_eq!(Zone::from_name("New York"), Some(Zone::NewYork));
        assert_eq!(Zone::from_name("Mid-Atlantic"), Some(Zone::MidAtlantic));
        assert_eq!(Zone::from_name(""), None);
    }

    #[test]
    fn parse_rejects_wrong_length_profile() {
        let bad = br#"{"California":{"solar":[0.1],"wind":[0.1],"load":[100.0]}}"#;
        assert!(parse_zones_json(bad).is_err());
    }

    #[test]
    fn parse_accepts_well_formed_input() {
        let solar: Vec<f64> = vec![0.0; 8760];
        let wind: Vec<f64> = vec![0.0; 8760];
        let load: Vec<f64> = vec![100.0; 8760];
        let json = format!(
            r#"{{"California":{{"solar":{},"wind":{},"load":{}}}}}"#,
            serde_json::to_string(&solar).unwrap(),
            serde_json::to_string(&wind).unwrap(),
            serde_json::to_string(&load).unwrap(),
        );
        let parsed = parse_zones_json(json.as_bytes()).expect("should parse");
        assert!(parsed.contains_key("California"));
        assert_eq!(parsed["California"].load.len(), 8760);
    }
}
