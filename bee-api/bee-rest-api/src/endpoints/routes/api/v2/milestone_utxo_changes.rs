// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_ledger::types::OutputDiff;
use bee_message::milestone::MilestoneIndex;
use bee_storage::access::Fetch;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::responses::UtxoChangesResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/milestones/:milestone_index/utxo-changes",
        get(milestone_utxo_changes::<B>),
    )
}

pub(crate) async fn milestone_utxo_changes<B: StorageBackend>(
    Path(milestone_index): Path<MilestoneIndex>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let fetched = Fetch::<MilestoneIndex, OutputDiff>::fetch(&*args.storage, &milestone_index)
        .map_err(|_| ApiError::ServiceUnavailable("can not fetch from storage".to_string()))?
        .ok_or_else(|| ApiError::NotFound("can not find Utxo changes for given milestone index".to_string()))?;

    Ok(Json(UtxoChangesResponse {
        index: *milestone_index,
        created_outputs: fetched.created_outputs().iter().map(|id| id.to_string()).collect(),
        consumed_outputs: fetched.consumed_outputs().iter().map(|id| id.to_string()).collect(),
    }))
}