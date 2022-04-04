// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_storage::access::AsIterator;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::{dtos::ReceiptDto, responses::ReceiptsResponse},
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/receipts", get(receipts::<B>))
}

pub(crate) async fn receipts<B: StorageBackend>(
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let mut receipts_dto = Vec::new();
    let iterator =
        AsIterator::<(MilestoneIndex, Receipt), ()>::iter(&*args.storage).map_err(|_| ApiError::InternalError)?;

    for result in iterator {
        let ((_, receipt), _) = result.map_err(|_| ApiError::InternalError)?;
        receipts_dto.push(ReceiptDto::try_from(receipt).map_err(|_| ApiError::InternalError)?);
    }

    Ok(Json(ReceiptsResponse { receipts: receipts_dto }))
}
