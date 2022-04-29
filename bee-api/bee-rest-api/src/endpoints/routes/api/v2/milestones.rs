// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::payload::milestone::{MilestoneId, MilestoneIndex};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::{ROUTE_MILESTONE_BY_MILESTONE_ID, ROUTE_MILESTONE_BY_MILESTONE_INDEX},
        filters::with_tangle,
        path_params::{milestone_id, milestone_index},
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::responses::MilestoneResponse,
};

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    super::path().and(
        warp::path("milestones")
            .and(
                milestone_id()
                    .and(warp::path::end())
                    .and(warp::get())
                    .and(has_permission(
                        ROUTE_MILESTONE_BY_MILESTONE_ID,
                        public_routes.clone(),
                        allowed_ips.clone(),
                    ))
                    .and(with_tangle(tangle.clone()))
                    .and_then(|milestone_id, tangle| async move { milestone_by_milestone_id(milestone_id, tangle) })
                    .boxed(),
            )
            .or(warp::path("by-index")
                .and(milestone_index())
                .and(warp::path::end())
                .and(warp::get())
                .and(has_permission(
                    ROUTE_MILESTONE_BY_MILESTONE_INDEX,
                    public_routes.clone(),
                    allowed_ips.clone(),
                ))
                .and(with_tangle(tangle.clone()))
                .and_then(
                    |milestone_index, tangle| async move { milestone_by_milestone_index(milestone_index, tangle) },
                )
                .boxed()),
    )
}

pub(crate) fn milestone_by_milestone_index<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    let milestone_id = match tangle.get_milestone_metadata(milestone_index) {
        Some(milestone_metadata) => *milestone_metadata.milestone_id(),
        None => return Err(reject::custom(CustomRejection::NotFound("data not found".to_string()))),
    };

    match tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => Ok(warp::reply::json(&MilestoneResponse((&milestone_payload).into()))),
        None => Err(reject::custom(CustomRejection::NotFound("data not found".to_string()))),
    }
}

pub(crate) fn milestone_by_milestone_id<B: StorageBackend>(
    milestone_id: MilestoneId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => Ok(warp::reply::json(&MilestoneResponse((&milestone_payload).into()))),
        None => Err(reject::custom(CustomRejection::NotFound("data not found".to_string()))),
    }
}
