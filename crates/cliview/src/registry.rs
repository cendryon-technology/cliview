use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::handler::{ActionHandler, PageHandler, StreamHandler};

pub(crate) struct Registry {
    pub app_name: String,
    pub pages: BTreeMap<String, PageEntry>,
    pub actions: BTreeMap<String, ActionEntry>,
    pub streams: BTreeMap<String, StreamEntry>,
}

pub(crate) struct PageEntry {
    pub title: String,
    pub handler: Arc<dyn PageHandler>,
}

pub(crate) struct ActionEntry {
    pub title: String,
    pub handler: Arc<dyn ActionHandler>,
}

pub(crate) struct StreamEntry {
    pub title: String,
    pub handler: Arc<dyn StreamHandler>,
}

impl Registry {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            pages: BTreeMap::new(),
            actions: BTreeMap::new(),
            streams: BTreeMap::new(),
        }
    }

    pub fn meta(&self) -> Meta {
        Meta {
            name: self.app_name.clone(),
            pages: self
                .pages
                .iter()
                .map(|(id, e)| MetaItem {
                    id: id.clone(),
                    title: e.title.clone(),
                    input_schema: None,
                })
                .collect(),
            actions: self
                .actions
                .iter()
                .map(|(id, e)| MetaItem {
                    id: id.clone(),
                    title: e.title.clone(),
                    input_schema: Some(e.handler.input_schema()),
                })
                .collect(),
            streams: self
                .streams
                .iter()
                .map(|(id, e)| MetaItem {
                    id: id.clone(),
                    title: e.title.clone(),
                    input_schema: None,
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Meta {
    pub name: String,
    pub pages: Vec<MetaItem>,
    pub actions: Vec<MetaItem>,
    pub streams: Vec<MetaItem>,
}

#[derive(Serialize)]
pub(crate) struct MetaItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}
