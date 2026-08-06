//! `NewExtendedMiningJob` (M-NEMJ-*).

use stratum_apps::stratum_core::{
    mining_sv2::{
        OpenExtendedMiningChannel, MESSAGE_TYPE_NEW_EXTENDED_MINING_JOB,
        MESSAGE_TYPE_OPEN_EXTENDED_MINING_CHANNEL_SUCCESS,
    },
    parsers_sv2::{AnyMessage, Mining},
};

use super::super::ScenarioResult;
use crate::{client::TestClient, endpoint::Endpoint, scenarios::ScenarioFn};

// ---------------------------------------------------------------------------
// M-NEMJ-1 — first post-open message is `NewExtendedMiningJob`
// ---------------------------------------------------------------------------

/// §5.3.16: the first message after an extended channel opens MUST be a
/// `NewExtendedMiningJob`.
///
/// Covers: M-NEMJ-1
pub async fn first_message_after_open_is_job(endpoint: Endpoint) -> ScenarioResult {
    let client = TestClient::connect(&endpoint).await?;

    let open = OpenExtendedMiningChannel {
        request_id: 1,
        user_identity: endpoint.user_identity.clone().try_into().unwrap(),
        nominal_hash_rate: 1_000_000.0,
        max_target: vec![0xff_u8; 32].try_into().unwrap(),
        min_extranonce_size: 4,
    };
    client
        .send(AnyMessage::Mining(Mining::OpenExtendedMiningChannel(open)))
        .await?;

    // drain the success frame
    let _ = client
        .expect_from_server(MESSAGE_TYPE_OPEN_EXTENDED_MINING_CHANNEL_SUCCESS)
        .await?;

    let (msg_type, _) = client.next_from_server().await?;
    if msg_type != MESSAGE_TYPE_NEW_EXTENDED_MINING_JOB {
        return Err(format!(
            "expected NewExtendedMiningJob (0x{MESSAGE_TYPE_NEW_EXTENDED_MINING_JOB:02x}) after extended channel open, got 0x{msg_type:02x}"
        )
        .into());
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// registry entries for this module
// ---------------------------------------------------------------------------

pub fn entries() -> Vec<(&'static str, ScenarioFn)> {
    vec![("M-NEMJ-1 first-message-after-open-is-job", |e| {
        Box::pin(first_message_after_open_is_job(e))
    })]
}
