// https://github.com/extrawurst/gitui/blob/master/src/ui/style.rs

use std::rc::Rc;

use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};
use color_eyre::Result;

pub type SharedTheme = Rc<Theme>;


mod color_parser {
    use serde::{Serializer, Deserializer};
    use std::str::FromStr;

    use super::*;

    pub fn serialize<S>(c: &Color, serializer: S) -> Result<S::Ok, S::Error> 
    where 
        S: Serializer
    {
        let s = c.to_string();
        String::serialize(&s, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Color::from_str(&s).map_err(serde::de::Error::custom)
    }
    
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Theme {

    #[serde(with = "color_parser")]
    border: Color,

    #[serde(with = "color_parser")]
    text: Color,

    #[serde(with = "color_parser")]
    fold: Color,

    #[serde(with = "color_parser")]
    cursor: Color,
}

impl Theme {

    pub fn border(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn text(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn fold(&self) -> Style {
        Style::default().fg(self.fold)
    }

    pub fn cursor(&self) -> Style {
        Style::default().bg(self.cursor)
    }
    
}

impl Default for Theme {
    fn default() -> Self {
        Theme { 
            border: Color::Rgb(49, 50, 68),
            text: Color::Rgb(205, 214, 244),
            fold: Color::Rgb(205, 214, 244),
            cursor: Color::Rgb(69, 71, 90),
        }
    }
}

