use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use interoperability_tests_sv2::{app_type::AppType, endpoint::Endpoint, runner::run_suite};

/// Regression test for https://github.com/stratum-mining/interoperability-tests-sv2/pull/2#issuecomment-5185463970
/// A dead endpoint must produce fast all-failed reports — no panic, no 60s stall.
#[tokio::test]
async fn dead_endpoint_fails_fast_without_panicking() {
    // a port nothing listens on
    let addr = SocketAddr::from(([127, 0, 0, 1], 9));
    let endpoint = Endpoint {
        addr,
        app_type: AppType::SoloPool,
        user_identity: "interoperability_tests_sv2".to_string(),
    };

    let start = Instant::now();
    let reports = tokio::time::timeout(Duration::from_secs(15), run_suite(&endpoint))
        .await
        .expect("run_suite stalled on a dead endpoint");
    assert!(
        start.elapsed() < Duration::from_secs(15),
        "dead endpoint took too long to report"
    );

    assert!(!reports.is_empty(), "no scenarios ran");
    for report in &reports {
        assert!(
            report.result.is_err(),
            "scenario {} must fail against a dead endpoint",
            report.id
        );
    }
}
