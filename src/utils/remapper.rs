use std::collections::HashMap;

pub struct Remapper {}

impl Remapper {
    pub fn remap(map: &HashMap<u32, i64>) -> HashMap<u32, proto::PropValue> {
        let mut hashmap = HashMap::<u32, proto::PropValue>::new();

        for (key, value) in map {
            let mut prop = proto::PropValue::default();
            prop.r#type = *key;
            prop.val = *value;
            prop.value = Some(proto::prop_value::Value::Ival(*value));
            hashmap.insert(*key, prop);
        }

        return hashmap;
    }

    pub fn remap2(map: &HashMap<u32, i64>) -> Vec<proto::PropPair> {
        let mut ret = vec![];

        for (key, value) in map {
            let mut prop = proto::PropValue::default();
            prop.r#type = *key;
            prop.val = *value;
            prop.value = Some(proto::prop_value::Value::Ival(*value));
            let mut pair = proto::PropPair::default();
            pair.r#type = *key;
            pair.prop_value = Some(prop);
            ret.push(pair);
        }

        return ret;
    }

    pub fn remap3(map: &HashMap<u32, f32>) -> Vec<proto::FightPropPair> {
        let mut ret = vec![];

        for (key, value) in map {
            let mut pair = proto::FightPropPair::default();
            pair.prop_type = *key;
            pair.prop_value = *value;
            ret.push(pair);
        }

        return ret;
    }
}