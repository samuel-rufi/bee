// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::broker::MqttBroker, storage::StorageBackend};

use bee_ledger::workers::event::MessageReferenced;
use bee_message::payload::Payload;
use bee_rest_api::types::{dtos::LedgerInclusionStateDto, responses::MessageMetadataResponse};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};
use bee_tangle::{ConflictReason, MsTangle};

use librumqttd::LinkTx;
use log::{debug, warn};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

pub(crate) fn spawn<N>(node: &mut N, mut messages_referenced_tx: LinkTx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let bus = node.bus();
    let tangle = node.resource::<MsTangle<N::Backend>>();
    let (tx, rx) = mpsc::unbounded_channel::<MessageReferenced>();

    node.spawn::<MqttBroker, _, _>(|shutdown| async move {
        debug!("MQTT 'messages/referenced' topic handler running.");

        let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

        while let Some(event) = receiver.next().await {
            // the message is newly referenced and therefore it's safe to unwrap
            let message = tangle.get(&event.message_id).await.map(|m| (*m).clone()).unwrap();
            // existing message <=> existing metadata, therefore it's safe to unwrap
            let metadata = tangle.get_metadata(&event.message_id).await.unwrap();
            // the message is referenced by a milestone therefore it's safe to unwrap
            let referenced_by_milestone_index = *metadata.milestone_index().unwrap();

            let (
                is_solid,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason,
                should_promote,
                should_reattach,
            ) = {
                let is_solid;
                let milestone_index;
                let ledger_inclusion_state;
                let conflict_reason;
                let should_promote;
                let should_reattach;

                is_solid = true;
                should_reattach = None;
                should_promote = None;

                // check if the message is an actual milestone
                if metadata.flags().is_milestone() {
                    milestone_index = Some(referenced_by_milestone_index);
                    ledger_inclusion_state = Some(LedgerInclusionStateDto::NoTransaction);
                    conflict_reason = None;
                } else {
                    milestone_index = None;
                    if let Some(Payload::Transaction(_)) = message.payload() {
                        if metadata.conflict() != ConflictReason::None {
                            ledger_inclusion_state = Some(LedgerInclusionStateDto::Conflicting);
                            conflict_reason = Some(metadata.conflict());
                        } else {
                            ledger_inclusion_state = Some(LedgerInclusionStateDto::Included);
                            conflict_reason = None;
                        }
                    } else {
                        ledger_inclusion_state = Some(LedgerInclusionStateDto::NoTransaction);
                        conflict_reason = None;
                    };
                }

                (
                    is_solid,
                    milestone_index,
                    ledger_inclusion_state,
                    conflict_reason,
                    should_reattach,
                    should_promote,
                )
            };

            let response = serde_json::to_string(&MessageMetadataResponse {
                message_id: (&event.message_id).to_string(),
                parent_message_ids: message.parents().iter().map(|id| id.to_string()).collect(),
                is_solid,
                referenced_by_milestone_index: Some(referenced_by_milestone_index),
                milestone_index,
                ledger_inclusion_state,
                conflict_reason: conflict_reason.map(|c| c as u8),
                should_promote,
                should_reattach,
            })
                .expect("error serializing to json");

            if let Err(e) = messages_referenced_tx.publish("messages/referenced", false, response) {
                warn!("Publishing MQTT message failed. Cause: {:?}", e);
            }

        }

        debug!("MQTT 'messages/referenced' topic handler stopped.");
    });

    bus.add_listener::<MqttBroker, _, _>(move |event: &MessageReferenced| {
        // The lifetime of the listeners is tied to the lifetime of the MQTT worker so they are removed together.
        // However, topic handlers are shutdown as soon as the signal is received, causing this send to potentially
        // fail and spam the output. The return is then ignored as not being essential.
        let _ = tx.send((*event).clone());
    });
}