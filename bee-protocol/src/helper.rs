// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::Heartbeat,
    peer::PeerManager,
    storage::StorageBackend,
    worker::{MessageRequesterWorkerEvent, MilestoneRequesterWorkerEvent, RequestedMessages, RequestedMilestones},
    ProtocolMetrics, Sender,
};

use bee_message::MessageId;
use bee_network::{NetworkController, PeerId};
use bee_tangle::{milestone::MilestoneIndex, MsTangle};

use log::warn;
use tokio::sync::mpsc;

// TODO move some functions to workers

// MilestoneRequest

pub(crate) async fn request_milestone<B: StorageBackend>(
    tangle: &MsTangle<B>,
    milestone_requester: &mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
    requested_milestones: &RequestedMilestones,
    index: MilestoneIndex,
    to: Option<PeerId>,
) {
    if !requested_milestones.contains_key(&index) && !tangle.contains_milestone(index).await {
        if let Err(e) = milestone_requester.send(MilestoneRequesterWorkerEvent(index, to)) {
            warn!("Requesting milestone failed: {}.", e);
        }
    }
}

pub(crate) async fn request_latest_milestone<B: StorageBackend>(
    tangle: &MsTangle<B>,
    milestone_requester: &mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
    requested_milestones: &RequestedMilestones,
    to: Option<PeerId>,
) {
    request_milestone(tangle, milestone_requester, requested_milestones, MilestoneIndex(0), to).await
}

// MessageRequest

pub(crate) async fn request_message<B: StorageBackend>(
    tangle: &MsTangle<B>,
    message_requester: &mpsc::UnboundedSender<MessageRequesterWorkerEvent>,
    requested_messages: &RequestedMessages,
    message_id: MessageId,
    index: MilestoneIndex,
) {
    if !tangle.contains(&message_id).await
        && !tangle.is_solid_entry_point(&message_id)
        && !requested_messages.contains_key(&message_id)
    {
        if let Err(e) = message_requester.send(MessageRequesterWorkerEvent(message_id, index)) {
            warn!("Requesting message failed: {}.", e);
        }
    }
}

// Heartbeat

pub fn send_heartbeat(
    peer_manager: &PeerManager,
    network: &NetworkController,
    metrics: &ProtocolMetrics,
    to: PeerId,
    latest_solid_milestone_index: MilestoneIndex,
    pruning_milestone_index: MilestoneIndex,
    latest_milestone_index: MilestoneIndex,
) {
    Sender::<Heartbeat>::send(
        network,
        peer_manager,
        metrics,
        &to,
        Heartbeat::new(
            *latest_solid_milestone_index,
            *pruning_milestone_index,
            *latest_milestone_index,
            peer_manager.connected_peers(),
            peer_manager.synced_peers(),
        ),
    );
}

pub fn broadcast_heartbeat(
    peer_manager: &PeerManager,
    network: &NetworkController,
    metrics: &ProtocolMetrics,
    latest_solid_milestone_index: MilestoneIndex,
    pruning_milestone_index: MilestoneIndex,
    latest_milestone_index: MilestoneIndex,
) {
    peer_manager.for_each_peer(|peer_id, _| {
        send_heartbeat(
            peer_manager,
            network,
            metrics,
            peer_id.clone(),
            latest_solid_milestone_index,
            pruning_milestone_index,
            latest_milestone_index,
        )
    });
}