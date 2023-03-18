use std::time::SystemTime;
use std::convert::TryInto;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub struct TimeManager {
}

impl TimeManager {
    pub fn duration_since(time_point: SystemTime) -> u64 {
        SystemTime::now().duration_since(time_point).unwrap().as_millis().try_into().unwrap()
    }

    pub fn timestamp() -> u64 {
        return Self::duration_since(SystemTime::UNIX_EPOCH);
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Option<NaiveDateTime>, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;

        if time.is_empty() {
            Ok(None)
        } else {
            Ok(Some(NaiveDateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S").map_err(D::Error::custom)?))
        }
    }
}
