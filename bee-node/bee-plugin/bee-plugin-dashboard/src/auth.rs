// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use auth_helper::{
    jwt::{ClaimsBuilder, JsonWebToken},
    password,
};
use bee_gossip::{Keypair, PeerId};
use bee_rest_api::endpoints::auth::DASHBOARD_AUDIENCE_CLAIM;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use warp::{reject, Rejection, Reply};

use crate::{config::DashboardAuthConfig, rejection::CustomRejection};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub jwt: String,
}

pub(crate) fn auth(
    node_id: PeerId,
    keypair: Keypair,
    config: DashboardAuthConfig,
    body: JsonValue,
) -> Result<impl Reply, Rejection> {
    let jwt_json = &body["jwt"];

    if !jwt_json.is_null() {
        let jwt = JsonWebToken::from(
            jwt_json
                .as_str()
                .ok_or_else(|| reject::custom(CustomRejection::InvalidJwt))?
                .to_owned(),
        );
        return match jwt.validate(
            node_id.to_string(),
            config.user().to_owned(),
            DASHBOARD_AUDIENCE_CLAIM.to_owned(),
            true,
            keypair.secret().as_ref(),
        ) {
            Ok(_) => Ok(warp::reply::json(&AuthResponse { jwt: jwt.to_string() })),
            Err(_) => Err(reject::custom(CustomRejection::InvalidJwt)),
        };
    }

    let user_json = &body["user"];

    let user = if user_json.is_null() {
        return Err(reject::custom(CustomRejection::NoUserProvided));
    } else {
        user_json
            .as_str()
            .ok_or_else(|| reject::custom(CustomRejection::InvalidCredentials))?
    };

    if user != config.user() {
        return Err(reject::custom(CustomRejection::InvalidCredentials));
    }

    let password_json = &body["password"];

    let password = if password_json.is_null() {
        return Err(reject::custom(CustomRejection::NoPasswordProvided));
    } else {
        password_json
            .as_str()
            .ok_or_else(|| reject::custom(CustomRejection::InvalidCredentials))?
    };

    if !password::password_verify(
        password.as_bytes(),
        &hex::decode(config.password_salt()).unwrap(),
        &hex::decode(config.password_hash()).unwrap(),
    )
    .map_err(|e| reject::custom(CustomRejection::InternalError(e.to_string())))?
    {
        return Err(reject::custom(CustomRejection::InvalidCredentials));
    }

    let claims = ClaimsBuilder::new(
        node_id.to_string(),
        config.user().to_owned(),
        DASHBOARD_AUDIENCE_CLAIM.to_owned(),
    )
    .with_expiry(config.session_timeout())
    .build()
    .map_err(|e| reject::custom(CustomRejection::InternalError(e.to_string())))?;

    let jwt = JsonWebToken::new(claims, keypair.secret().as_ref())
        .map_err(|e| reject::custom(CustomRejection::InternalError(e.to_string())))?;

    Ok(warp::reply::json(&AuthResponse { jwt: jwt.to_string() }))
}
