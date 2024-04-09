use derive_setters::Setters;
use serde::Serialize;

#[derive(serde::Serialize, Default, Setters, Debug, Clone)]
pub struct Message {
    blocks: Vec<Block>,
}

#[allow(dead_code)]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Actions,
    Text,
    Section,
    Divider,
    Mrkdwn,
    PlainText,
    Button,
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(serde::Serialize, Default, Setters, Debug, Clone)]
pub struct Block {
    #[serde(rename = "type")]
    _type: BlockType,
    text: Option<TextBlock>,
    elements: Option<Vec<Button>>,
}

impl Block {
    pub fn section(t: BlockType, text: &str) -> Self {
        Self {
            _type: BlockType::Text,
            text: Some(TextBlock {
                _type: t,
                text: text.into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn divider() -> Self {
        Self {
            text: None,
            _type: BlockType::Divider,
            ..Default::default()
        }
    }

    pub fn actions(items: Vec<Button>) -> Self {
        Self {
            _type: BlockType::Actions,
            elements: Some(items),
            ..Default::default()
        }
    }
}

#[derive(serde::Serialize, Default, Setters, Debug, Clone)]
struct TextBlock {
    #[serde(rename = "type")]
    _type: BlockType,
    text: String,
    emoji: bool,
}

impl TextBlock {
    pub fn plain(content: &str) -> Self {
        Self {
            _type: BlockType::PlainText,
            text: content.into(),
            ..Default::default()
        }
    }
}

#[derive(serde::Serialize, Setters, Default, Debug, Clone)]
pub struct Button {
    #[serde(rename = "type")]
    _type: BlockType,
    url: Option<String>,
    text: TextBlock,
}

impl Button {
    pub fn link(href: &str, label: &str) -> Self {
        Self {
            _type: BlockType::Button,
            url: Some(href.into()),
            text: TextBlock::plain(label),
        }
    }
}
