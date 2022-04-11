// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_storage::access::Fetch;
use log::error;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::{dtos::ReceiptDto, responses::ReceiptsResponse},
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/receipts/:milestone_index", get(receipts_at::<B>))
}

pub(crate) async fn receipts_at<B: StorageBackend>(
    Path(milestone_index): Path<MilestoneIndex>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let mut receipts_dto = Vec::new();

    if let Some(receipts) =
        Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&*args.storage, &milestone_index).map_err(|e| {
            error!("cannot fetch from storage: {}", e);
            ApiError::InternalError
        })?
    {
        for receipt in receipts {
            receipts_dto.push(ReceiptDto::from(receipt));
        }
    }

    Ok(Json(ReceiptsResponse { receipts: receipts_dto }))
}
