use serde::{Deserialize, Serialize};

use crate::{CharStringExt, TokenKind};

/// A [`Token`](crate::Token) that holds its content as a fat [`Vec<char>`] rather than as a
/// [`Span`](crate::Span).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Hash, Eq)]
pub struct FatToken {
    pub content: Vec<char>,
    pub kind: TokenKind,
}

impl From<FatStringToken> for FatToken {
    fn from(value: FatStringToken) -> Self {
        Self {
            content: value.content.chars().collect(),
            kind: value.kind,
        }
    }
}

/// Similar to a [`FatToken`], but uses a [`String`] as the underlying store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Hash, Eq)]
pub struct FatStringToken {
    pub content: String,
    pub kind: TokenKind,
}

impl From<FatToken> for FatStringToken {
    fn from(value: FatToken) -> Self {
        Self {
            content: value.content.to_string(),
            kind: value.kind,
        }
    }
}
