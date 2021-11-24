use std::fs::read_to_string; // use instead of std::fs::File
use std::path::Path;
use std::collections::HashMap;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::avatar_skill_depot::AvatarSkillDepot;

struct JsonReader {
    base_path: String,
}

pub struct JsonManager {
    reader: JsonReader,
    pub avatar_skill_depot: HashMap<u32,AvatarSkillDepot>,
}

impl JsonManager {
    pub fn new(directory: &str) -> JsonManager {
        let reader = JsonReader::new(directory);

        let asd: Vec<AvatarSkillDepot> = reader.read_json_list("AvatarSkillDepot");

        return JsonManager {
            reader: reader,
            avatar_skill_depot: asd.into_iter().map(|a| (a.id, a)).collect(),
        };
    }
}

impl JsonReader {
    pub fn new(directory: &str) -> JsonReader {
        return JsonReader {
            base_path: directory.to_owned(),
        };
    }

    fn read_json_list<T>(&self, name: &str) -> Vec<T>
        where T: DeserializeOwned
    {
        let path = format!("{}/{}ExcelConfigData.json", self.base_path, name);

        let json_file_path = Path::new(&path);
        let json_file_str = read_to_string(json_file_path).unwrap_or_else(|_| panic!("File {} not found", path));
        let data: Vec<T> = serde_json::from_str(&json_file_str).expect("Error while reading json");
        return data;
    }
}
