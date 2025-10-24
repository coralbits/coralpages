// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Page not found: {path}")]
    PageNotFound { path: String },
    #[error("Store not found: {store}")]
    StoreNotFound { store: String },
    #[error("Invalid path: {path}")]
    InvalidPath { path: String },
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl StoreError {
    pub fn error_code(&self) -> &'static str {
        match self {
            StoreError::PageNotFound { .. } => "PAGE_NOT_FOUND",
            StoreError::StoreNotFound { .. } => "STORE_NOT_FOUND",
            StoreError::InvalidPath { .. } => "INVALID_PATH",
            StoreError::Internal { .. } => "INTERNAL_ERROR",
        }
    }

    pub fn http_status(&self) -> u16 {
        match self {
            StoreError::PageNotFound { .. } => 404,
            StoreError::StoreNotFound { .. } => 404,
            StoreError::InvalidPath { .. } => 400,
            StoreError::Internal { .. } => 500,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub details: String,
    pub code: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<String>,
}

impl ErrorResponse {
    pub fn from_store_error(error: &StoreError) -> Self {
        match error {
            StoreError::PageNotFound { path } => ErrorResponse {
                details: format!("Page '{}' not found", path),
                code: error.error_code().to_string(),
                status: error.http_status(),
                path: Some(path.clone()),
                store: None,
            },
            StoreError::StoreNotFound { store } => ErrorResponse {
                details: format!("Store '{}' not found", store),
                code: error.error_code().to_string(),
                status: error.http_status(),
                path: None,
                store: Some(store.clone()),
            },
            StoreError::InvalidPath { path } => ErrorResponse {
                details: format!("Invalid path format: {}", path),
                code: error.error_code().to_string(),
                status: error.http_status(),
                path: Some(path.clone()),
                store: None,
            },
            StoreError::Internal { message } => ErrorResponse {
                details: message.clone(),
                code: error.error_code().to_string(),
                status: error.http_status(),
                path: None,
                store: None,
            },
        }
    }
}

pub struct ResultI<T> {
    pub count: usize,
    pub results: Vec<T>,
}
