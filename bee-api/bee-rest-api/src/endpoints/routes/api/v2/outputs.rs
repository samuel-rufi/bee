// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_ledger::{
    types::{ConsumedOutput, CreatedOutput, LedgerIndex},
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_message::output::OutputId;
use bee_storage::access::Fetch;
use futures::channel::oneshot;
use log::error;

use crate::{
    endpoints::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode},
    types::responses::OutputResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/outputs/:output_id", get(outputs::<B>))
}

async fn outputs<B: StorageBackend>(
    CustomPath(output_id): CustomPath<OutputId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>();

    if let Err(e) = args
        .consensus_worker
        .send(ConsensusWorkerCommand::FetchOutput(output_id, cmd_tx))
    {
        error!("request to consensus worker failed: {}", e);
        return Err(ApiError::InternalError);
    }

    let consensus_worker_response = cmd_rx.await.map_err(|e| {
        error!("response from consensus worker failed: {}", e);
        ApiError::InternalError
    })?;

    match consensus_worker_response {
        (Ok(response), ledger_index) => match response {
            Some(output) => {
                let consumed_output =
                    Fetch::<OutputId, ConsumedOutput>::fetch(&*args.storage, &output_id).map_err(|e| {
                        error!("cannot fetch from storage: {}", e);
                        ApiError::InternalError
                    })?;

                Ok(Json(OutputResponse {
                    message_id: output.message_id().to_string(),
                    transaction_id: output_id.transaction_id().to_string(),
                    output_index: output_id.index(),
                    is_spent: consumed_output.is_some(),
                    // TODO: replace with the milestone index for which the output was spent
                    milestone_index_spent: None,
                    // TODO: replace with the timestamp of the milestone for which the output was spent
                    milestone_timestamp_spent: None,
                    // TODO: replace with the transaction id that spent the output
                    transaction_id_spent: None,
                    // TODO: replace with the milestone index that booked the output
                    milestone_index_booked: 0,
                    // TODO: replace with the timestamp of the milestone that booked the output
                    milestone_timestamp_booked: 0,
                    ledger_index: *ledger_index,
                    output: output.inner().into(),
                }))
            }

            None => Err(ApiError::NotFound),
        },
        (Err(e), _) => {
            error!("response from consensus worker failed: {}", e);
            Err(ApiError::InternalError)
        }
    }
}