use chrono::NaiveDateTime;
use uuid::Uuid;
use std::{collections::HashMap, process::Command};

use serde_json::Value;
use serde::{Deserialize, Serialize};
use color_eyre::Result;


mod date_parser {
    use serde::{Serializer, Deserializer};
    use super::*;

    const FORMAT: &str = "%Y%m%dT%H%M%SZ";

    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {

        let s = dt.format(FORMAT).to_string();
        String::serialize(&s, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }

}

mod optional_date_parser {

    use serde::{Serializer, Deserializer};
    use super::*;

    const FORMAT: &str = "%Y%m%dT%H%M%SZ";
    pub fn serialize<S>(dt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => {
                let s = dt.format(FORMAT).to_string();
                String::serialize(&s, serializer)
            },
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Ok(Some(NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?)),
            None => Ok(None),
        }
    }
}

mod uuid_parser {
    use serde::{Serializer, Deserializer};
    use super::*;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = uuid.to_string();
        String::serialize(&s, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }

}

mod optional_uuid_parser {
    use serde::{Serializer, Deserializer};
    use super::*;

    pub fn serialize<S>(uuid: &Option<Uuid>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match uuid {
            Some(u) => {
                let s = u.to_string();
                String::serialize(&s, serializer)
            },
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Ok(Some(Uuid::parse_str(&s).map_err(serde::de::Error::custom)?)),
            None => Ok(None),
        }
    }

}

fn default_time() -> Option<NaiveDateTime> {
    None
}

fn default_uuid() -> Option<Uuid> {
    None
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Annotation {
    #[serde(with = "date_parser")]
    entry: NaiveDateTime,

    description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: u32,

    #[serde(with = "uuid_parser")]
    pub uuid: Uuid,

    pub description: String,

    #[serde(with = "date_parser")]
    pub entry: NaiveDateTime,

    #[serde(with = "date_parser")]
    pub modified: NaiveDateTime,

    #[serde(with = "optional_date_parser", default = "default_time")]
    pub due: Option<NaiveDateTime>,

    #[serde(with = "optional_date_parser", default = "default_time")]
    pub start: Option<NaiveDateTime>,

    #[serde(with = "optional_date_parser", default = "default_time")]
    pub end: Option<NaiveDateTime>,

    pub status: String,

    #[serde(default = "Vec::new")]
    pub tags: Vec<String>,

    pub urgency: f32,

    pub project: Option<String>,

    pub mask: Option<String>,

    #[serde(rename = "imask")]
    pub mask_index: Option<u32>,

    #[serde(with = "optional_uuid_parser", default = "default_uuid")]
    pub parent: Option<Uuid>,

    #[serde(with = "optional_uuid_parser", default = "default_uuid")]
    pub sub_of: Option<Uuid>,

    #[serde(default = "Vec::new")]
    pub annotations: Vec<Annotation>,

    #[serde(flatten)]
    pub udas: HashMap<String, Value>,
}

pub fn from_json(val: Value) -> Result<HashMap<Uuid, Task>> {
    let mut task_map: HashMap<Uuid, Task> = HashMap::default();
    for el in val.as_array().unwrap() {
        let task: Task = serde_json::from_value(el.clone())?;
        task_map.insert(task.uuid, task);
    };
    Ok(task_map)
}

pub fn get_tasks(filter: Option<&str>) -> Result<HashMap<Uuid, Task>> {
    let output = if filter.is_some() {
        Command::new("task").arg(filter.unwrap()).arg("export").output()?
    } else {
        Command::new("task").arg("export").output()?
    };
    let contents = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&contents)?;
    let result = from_json(json)?;
    Ok(result)
}

