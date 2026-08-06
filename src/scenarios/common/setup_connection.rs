//! Common Protocol Messages — `SetupConnection` negotiation.
//!
//! Covers catalog rows C-SC-*, C-SCS-*, C-SCE-*.

use integration_tests_sv2::mock_roles::WithSetup;
use stratum_apps::stratum_core::{
    common_messages_sv2::{
        MESSAGE_TYPE_SETUP_CONNECTION_ERROR, MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS,
    },
    parsers_sv2::{AnyMessage, CommonMessages},
};

use crate::{
    client::{setup_connection, TestClient},
    endpoint::Endpoint,
    scenarios::ScenarioError,
};

use super::super::ScenarioResult;

// ---------------------------------------------------------------------------
// C-SCS-1 — framed `SetupConnection.Success` response
// ---------------------------------------------------------------------------

/// The endpoint MUST respond to `SetupConnection` with a properly framed
/// `.Success` — not silence, a wrong message type, or a TCP close.
///
/// Covers: C-SCS-1
pub async fn framed_setup_connection_response(endpoint: Endpoint) -> ScenarioResult {
    let client = TestClient::connect(&endpoint).await?;

    let (msg_type, msg) = client.next_from_server().await?;
    if msg_type != MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS {
        return Err(format!(
            "first message must be SetupConnectionSuccess, got type 0x{msg_type:02x}"
        )
        .into());
    }
    match msg {
        AnyMessage::Common(CommonMessages::SetupConnectionSuccess(m)) => {
            Ok(Some(format!("used_version={}", m.used_version)))
        }
        other => Err(format!("expected SetupConnectionSuccess, got {other}").into()),
    }
}

// ---------------------------------------------------------------------------
// C-SCS-2 — `used_version` within negotiated range
// ---------------------------------------------------------------------------

/// The endpoint MUST honour the version range: `[2,2]` → `used_version == 2`;
/// `[3,3]` → framed `SetupConnectionError` (§3.6.2, §3.6.3).
///
/// Covers: C-SCS-2
pub async fn used_version_within_range(endpoint: Endpoint) -> ScenarioResult {
    let protocol = endpoint.app_type.protocol();
    let flags = endpoint.app_type.default_flags();

    // Probe 1 — min=max=2: server MUST negotiate version 2.
    let client = TestClient::connect(&endpoint).await?;
    let msg = client
        .expect_from_server(MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS)
        .await?;
    match msg {
        AnyMessage::Common(CommonMessages::SetupConnectionSuccess(m)) => {
            if m.used_version != 2 {
                return Err(format!(
                    "used_version must be 2 with [2,2] range, got {}",
                    m.used_version
                )
                .into());
            }
        }
        other => {
            return Err(format!("expected SetupConnectionSuccess for [2,2], got {other}").into())
        }
    }

    // Probe 2 — min=max=3: server MUST reject (no common version).
    let setup = setup_connection(protocol, 3, 3, flags);
    let client = TestClient::connect_with_setup(&endpoint, WithSetup::Yes(setup)).await?;
    let (msg_type, msg) = client.next_from_server().await?;
    match msg {
        AnyMessage::Common(CommonMessages::SetupConnectionError(_)) => {}
        AnyMessage::Common(CommonMessages::SetupConnectionSuccess(m)) => {
            return Err(format!(
                "endpoint accepted unsatisfiable version range [3,3] with used_version={}",
                m.used_version
            )
            .into())
        }
        other => {
            return Err(format!(
                "expected SetupConnectionError for [3,3] range, got type 0x{msg_type:02x}: {other}"
            )
            .into())
        }
    }

    Ok(Some("[2,2] accepted; [3,3] rejected".into()))
}

// ---------------------------------------------------------------------------
// C-SC-7 — unsupported protocol rejected
// ---------------------------------------------------------------------------

/// The endpoint MUST reject a `SetupConnection` for a sub-protocol it does
/// not serve with a framed `SetupConnectionError`.
///
/// Covers: C-SC-7
pub async fn unsupported_protocol_rejected(endpoint: Endpoint) -> ScenarioResult {
    let setup = setup_connection(endpoint.app_type.wrong_protocol(), 2, 2, 0);
    let client = TestClient::connect_with_setup(&endpoint, WithSetup::Yes(setup)).await?;

    let error = expect_setup_connection_error(&client).await?;
    let code = error.error_code.as_utf8_or_hex();
    Ok(Some(format!("error_code=\"{code}\"")))
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

async fn expect_setup_connection_error(
    client: &TestClient,
) -> Result<
    stratum_apps::stratum_core::common_messages_sv2::SetupConnectionError<'static>,
    ScenarioError,
> {
    let msg = client
        .expect_from_server(MESSAGE_TYPE_SETUP_CONNECTION_ERROR)
        .await?;
    match msg {
        AnyMessage::Common(CommonMessages::SetupConnectionError(m)) => Ok(m),
        other => Err(format!("expected SetupConnectionError, got {other}").into()),
    }
}

// ---------------------------------------------------------------------------
// registry entries for this module
// ---------------------------------------------------------------------------

use crate::scenarios::ScenarioFn;

pub fn entries() -> Vec<(&'static str, ScenarioFn)> {
    vec![
        ("C-SCS-1 framed-setup-connection-response", |e| {
            Box::pin(framed_setup_connection_response(e))
        }),
        ("C-SCS-2 used-version-within-range", |e| {
            Box::pin(used_version_within_range(e))
        }),
        ("C-SC-7 unsupported-protocol-rejected", |e| {
            Box::pin(unsupported_protocol_rejected(e))
        }),
    ]
}
