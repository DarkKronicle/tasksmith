use std::error::Error;
use std::fmt;
use chrono::NaiveDateTime;
use uuid::Uuid;
use std::process::Command;
use serde_json;
use serde_json::Value;
use color_eyre::Result;


#[derive(Clone, Debug)]
pub struct Annotation {
    entry: NaiveDateTime,
    description: String,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: u64,
    pub uuid: Uuid,
    pub description: String,
    pub entry: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub due: Option<NaiveDateTime>,
    pub start: Option<NaiveDateTime>,
    pub end: Option<NaiveDateTime>,
    pub status: String,
    pub tags: Vec<String>,
    pub urgency: f64,
    pub project: Option<String>,
    pub mask: Option<String>,
    pub mask_index: Option<u64>,
    pub parent: Option<Uuid>,
    pub annotations: Vec<Annotation>,
}

#[derive(Clone, Debug)]
pub struct Tasks {
    pub tasks: Vec<Task>,
    pub filter: Option<String>,
}

#[derive(Debug)]
pub enum TaskParseError {
    MissingField(String),
    InvalidTime,
    InvalidType(String),
    InvalidUuid,
    CommandError(String),
    JsonError(String),
}

impl fmt::Display for TaskParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            TaskParseError::MissingField(v) => write!(f, "Error parsing task. Missing field: {}", v),
            TaskParseError::InvalidType(v) => write!(f, "Error parsing task. Invalid type on field: {}", v),
            TaskParseError::InvalidTime => write!(f, "Error parsing task. Invalid time format."),
            TaskParseError::InvalidUuid => write!(f, "Error parsing task. Invalid uuid format."),
            TaskParseError::CommandError(err) => write!(f, "Error running task command. {}", err),
            TaskParseError::JsonError(err) => write!(f, "Error deserializing task json. {}", err),
        }
    }
}

impl Error for TaskParseError {

}

impl Tasks {

    pub fn from_json(filter: Option<String>, val: Value) -> Result<Tasks, TaskParseError> {
        let mut task_list: Vec<Task> = Vec::new();
        for el in val.as_array().unwrap() {
            task_list.push(Task {
                id: get_u64(&el, "id")?.ok_or(TaskParseError::MissingField("id".to_string()))?,

                uuid: parse_uuid(&el, "uuid")?.ok_or(TaskParseError::MissingField("uuid".to_string()))?,

                description: get_string(&el, "description")?.ok_or(TaskParseError::MissingField("description".to_string()))?,

                entry: parse_time(&el, "entry")?.ok_or(TaskParseError::MissingField("entry".to_string()))?,

                modified: parse_time(&el, "modified")?.ok_or(TaskParseError::MissingField("modified".to_string()))?,

                due: parse_time(&el, "due")?,
                start: parse_time(&el, "start")?,
                end: parse_time(&el, "end")?,

                status: get_string(&el, "status")?.ok_or(TaskParseError::MissingField("status".to_string()))?,

                tags: get_string_vec(&el, "tags")?,

                urgency: get_f64(&el, "urgency")?.ok_or(TaskParseError::MissingField("urgency".to_string()))?,

                project: get_string(&el, "project")?,
                mask: get_string(&el, "mask")?,

                mask_index: get_u64(&el, "imask")?,
                parent: parse_uuid(&el, "parent")?,
                annotations: get_annotation_vec(&el, "annotations")?,
            });
        };
        Ok(Tasks { tasks: task_list, filter })
    }

}


fn get_annotation_vec<'a>(val: &'a Value, key: &'a str) -> Result<Vec<Annotation>, TaskParseError> {
    let value = get_value(val, key);
    match value {
        Some(v) => v.as_array().ok_or(TaskParseError::InvalidType(key.to_string())).map(|arr| {
            // FIX: do not unwrap here!
            arr.iter().map(|el| Annotation {
                entry: parse_time(&el, "entry").unwrap().unwrap(),
                description: get_string(&el, "entry").unwrap().unwrap(),
            }).collect()
        }),
        None => Ok(vec![]),
    }
}


fn get_string_vec<'a>(val: &'a Value, key: &'a str) -> Result<Vec<String>, TaskParseError> {
    let value = get_value(val, key);
    match value {
        Some(v) => v.as_array().ok_or(TaskParseError::InvalidType(key.to_string())).map(|arr| {
            // FIX: do not unwrap here!
            arr.iter().map(|el| el.as_str().map(|s| s.to_string()).unwrap()).collect()
        }),
        None => Ok(vec![]),
    }
}


fn get_f64<'a>(val: &'a Value, key: &'a str) -> Result<Option<f64>, TaskParseError> {
    let value = get_value(val, key);
    match value {
        Some(v) => v.as_f64().ok_or(TaskParseError::InvalidType(key.to_string())).map(|s| Some(s)),
        None => Ok(None),
    }
}

fn get_u64<'a>(val: &'a Value, key: &'a str) -> Result<Option<u64>, TaskParseError> {
    let value = get_value(val, key);
    match value {
        Some(v) => v.as_u64().ok_or(TaskParseError::InvalidType(key.to_string())).map(|s| Some(s)),
        None => Ok(None),
    }
}

fn get_string<'a>(val: &'a Value, key: &'a str) -> Result<Option<String>, TaskParseError> {
    let value = get_value(val, key);
    match value {
        Some(v) => v.as_str().ok_or(TaskParseError::InvalidType(key.to_string())).map(|s| Some(s.to_string())),
        None => Ok(None),
    }
}

fn get_value(val: &Value, key: &str) -> Option<Value> {
    let el_value = &val[key];
    if el_value == &Value::Null {
        return None;
    }
    return Some(el_value.clone());
}

fn parse_time(val: &Value, key: &str) -> Result<Option<NaiveDateTime>, TaskParseError> {
    let contents = get_string(val, key)?;
    match contents {
        Some(s) => Ok(Some(NaiveDateTime::parse_from_str(&s, "%Y%m%dT%H%M%SZ").map_err(|_| TaskParseError::InvalidTime)?)),
        None => Ok(None),
    }
}

fn parse_uuid(val: &Value, key: &str) -> Result<Option<Uuid>, TaskParseError> {
    let contents = get_string(val, key)?;
    match contents {
        Some(s) => Ok(Some(Uuid::parse_str(&s).map_err(|_| TaskParseError::InvalidUuid)?)),
        None => Ok(None),
    }

}



pub fn get_tasks(filter: Option<&str>) -> Result<Tasks, TaskParseError> {
    let output = if filter.is_some() {
        Command::new("task").arg(filter.unwrap()).arg("export").output().map_err(|err| TaskParseError::CommandError(err.to_string()))?
    } else {
        Command::new("task").arg("export").output().map_err(|err| TaskParseError::CommandError(err.to_string()))?
    };
    let contents = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&contents).map_err(|err| TaskParseError::JsonError(err.to_string()))?;
    let result = Tasks::from_json(filter.map(|s| s.to_string()), json)?;
    Ok(result)
}
