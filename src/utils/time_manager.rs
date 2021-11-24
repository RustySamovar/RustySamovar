use std::time::SystemTime;
use std::convert::TryInto;

pub struct TimeManager {
}

impl TimeManager {
    pub fn duration_since(time_point: SystemTime) -> u64 {
        SystemTime::now().duration_since(time_point).unwrap().as_millis().try_into().unwrap()
    }

    pub fn timestamp() -> u64 {
        return Self::duration_since(SystemTime::UNIX_EPOCH);
    }
}
