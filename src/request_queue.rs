use std::collections::VecDeque;

use crate::{http_request::HttpRequest, http_response::HttpResponse};

#[derive(Debug)]
pub struct RequestQueue {
    pub requests: VecDeque<HttpRequest>,
    pub max_queued: usize,
}

impl RequestQueue {
    pub fn new(max_queued: usize) -> Self {
        Self {
            requests: VecDeque::new(),
            max_queued,
        }
    }

    pub fn push(&mut self, request: HttpRequest) -> Result<(), HttpResponse> {
        if self.requests.len() >= self.max_queued {
            return Err(HttpResponse::service_unavailable(None));
        }
        self.requests.push_back(request);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<HttpRequest> {
        self.requests.pop_front()
    }
}