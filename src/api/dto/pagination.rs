//! Pagination-related DTOs for API requests and responses.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Query parameters for pagination.
#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationParams {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: u32,
    
    /// Number of items per page (max 100)
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

impl PaginationParams {
    /// Validates and normalizes pagination parameters.
    pub fn validate(mut self) -> Self {
        if self.page == 0 {
            self.page = 1;
        }
        if self.page_size == 0 || self.page_size > 100 {
            self.page_size = 20;
        }
        self
    }

    /// Calculates the offset for database queries.
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }

    /// Returns the limit for database queries.
    pub fn limit(&self) -> u32 {
        self.page_size
    }
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

/// Generic paged response wrapper.
#[derive(Debug, Serialize, ToSchema)]
pub struct PagedResponse<T> {
    /// The data items for this page
    pub data: Vec<T>,
    
    /// Pagination metadata
    pub pagination: PaginationMeta,
}

/// Pagination metadata.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    /// Current page number (1-based)
    pub page: u32,
    
    /// Number of items per page
    pub page_size: u32,
    
    /// Total number of items across all pages
    pub total_items: u64,
    
    /// Total number of pages
    pub total_pages: u32,
    
    /// Whether there is a next page
    pub has_next: bool,
    
    /// Whether there is a previous page
    pub has_prev: bool,
}

impl<T> PagedResponse<T> {
    /// Creates a new paged response.
    pub fn new(data: Vec<T>, params: &PaginationParams, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (params.page_size as f64)).ceil() as u32;
        let has_next = params.page < total_pages;
        let has_prev = params.page > 1;

        Self {
            data,
            pagination: PaginationMeta {
                page: params.page,
                page_size: params.page_size,
                total_items,
                total_pages,
                has_next,
                has_prev,
            },
        }
    }
}