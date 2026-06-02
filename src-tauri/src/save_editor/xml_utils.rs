use crate::save_editor::error::{Result, SaveEditorError};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;
use std::fs;

pub fn read_xml_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    if content.is_empty() {
        return Err(SaveEditorError::InvalidStructure(format!(
            "Save file is empty: {}",
            path.display()
        )));
    }
    Ok(content)
}

pub fn find_tag_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let value_start = start + open.len();
    let end = xml[value_start..].find(&close)? + value_start;
    Some(xml[value_start..end].to_string())
}

pub fn validate_save_root(xml: &str) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "SaveGame" {
                    return Ok(());
                } else {
                    return Err(SaveEditorError::InvalidStructure(format!(
                        "Expected <SaveGame> root, got <{}>",
                        name
                    )));
                }
            }
            Event::Eof => {
                return Err(SaveEditorError::InvalidStructure(
                    "No root element found".to_string(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}
