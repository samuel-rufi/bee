// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, response::IntoResponse, routing::get, Router};
use bee_message::payload::milestone::MilestoneId;

use crate::endpoints::{
    error::ApiError, extractors::path::CustomPath, routes::api::v2::milestones_by_milestone_index,
    storage::StorageBackend, ApiArgsFullNode,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/milestones/:milestone_id", get(milestones_by_milestone_id::<B>))
}

async fn milestones_by_milestone_id<B: StorageBackend>(
    CustomPath(milestone_id): CustomPath<MilestoneId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let milestone_index = match args.tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => milestone_payload.essence().index(),
        None => return Err(ApiError::NotFound),
    };

    milestones_by_milestone_index::milestones_by_milestone_index(CustomPath(milestone_index), Extension(args)).await
}