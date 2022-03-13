// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Bee REST API
//!
//! ## Feature Flags
//! - `cpt2`: Enable support for backwards compatible output and transaction payload types.

// #![deny(missing_docs, warnings)]

#![cfg_attr(doc_cfg, feature(doc_cfg))]

extern crate axum;

#[cfg(feature = "endpoints")]
pub mod endpoints;
pub mod types;
