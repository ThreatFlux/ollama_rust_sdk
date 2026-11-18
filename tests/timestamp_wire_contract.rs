//! Exercise the public timestamp models with real API JSON payloads.

use ollama_rust_sdk::Timestamp;
use ollama_rust_sdk::models::model_info::{Model, RunningModel};
use ollama_rust_sdk::types::HealthStatus;
use serde_json::json;

#[test]
fn api_timestamps_preserve_the_instant_and_nanoseconds() {
    let input = "2026-11-18T09:00:00.123456789+05:30";
    let expected = "2026-11-18T03:30:00.123456789Z";
    let model: Model = serde_json::from_value(json!({
        "name": "test", "size": 1, "digest": "sha256:test", "modified_at": input
    }))
    .expect("valid model JSON");
    let running: RunningModel = serde_json::from_value(json!({
        "name": "test", "size": 1, "digest": "sha256:test", "expires_at": input
    }))
    .expect("valid running-model JSON");
    let health: HealthStatus = serde_json::from_value(json!({
        "status": "ok", "timestamp": input
    }))
    .expect("valid health JSON");
    let instant: Timestamp = expected.parse().expect("valid UTC instant");
    assert_eq!(model.modified_at, Some(instant));
    assert_eq!(running.expires_at, Some(instant));
    assert_eq!(health.timestamp, Some(instant));
    assert_eq!(serde_json::to_value(model).unwrap()["modified_at"], expected);
    assert_eq!(serde_json::to_value(running).unwrap()["expires_at"], expected);
    assert_eq!(serde_json::to_value(health).unwrap()["timestamp"], expected);
}

#[test]
fn optional_timestamps_remain_missing_or_null() {
    for timestamp in [None, Some(serde_json::Value::Null)] {
        let mut payload = json!({"name": "test", "size": 1, "digest": "sha256:test"});
        if let Some(value) = timestamp {
            payload["modified_at"] = value;
        }
        let model: Model = serde_json::from_value(payload).expect("optional timestamp");
        assert_eq!(model.modified_at, None);
    }
    let health: HealthStatus = serde_json::from_value(json!({"status": "ok"})).unwrap();
    assert_eq!(health.timestamp, None);
    let running: RunningModel = serde_json::from_value(json!({
        "name": "test", "size": 1, "digest": "sha256:test", "expires_at": null
    }))
    .unwrap();
    assert_eq!(running.expires_at, None);
}

#[test]
fn invalid_or_ambiguous_api_timestamps_are_rejected() {
    for input in ["2026-13-18T09:00:00Z", "2026-11-18T09:00:00", "not-a-date"] {
        let result = serde_json::from_value::<HealthStatus>(json!({
            "status": "ok", "timestamp": input
        }));
        assert!(result.is_err(), "accepted invalid timestamp: {input}");
    }
}

#[test]
fn leap_seconds_follow_the_documented_unix_timescale() {
    let model: Model = serde_json::from_value(json!({
        "name": "test", "size": 1, "digest": "sha256:test",
        "modified_at": "2016-12-31T23:59:60.123456789Z"
    }))
    .expect("Jiff normalizes the unsupported leap second");
    assert_eq!(
        serde_json::to_value(model).unwrap()["modified_at"],
        "2016-12-31T23:59:59.123456789Z"
    );
}

#[test]
fn running_model_expiry_uses_the_current_utc_clock() {
    let mut model: RunningModel = serde_json::from_value(json!({
        "name": "test", "size": 1, "digest": "sha256:test"
    }))
    .unwrap();
    assert!(!model.expires_soon());
    model.expires_at = Some(Timestamp::now() - jiff::SignedDuration::from_secs(300));
    assert!(model.expires_soon());
    model.expires_at = Some(Timestamp::now() + jiff::SignedDuration::from_secs(300));
    assert!(!model.expires_soon());
}

#[tokio::test]
async fn cli_renders_a_models_timestamp_from_the_api() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "models": [{"name": "test", "size": 1, "digest": "sha256:test",
                "modified_at": "2026-11-18T09:00:00+05:30"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let url = server.uri();
    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new(env!("CARGO_BIN_EXE_ollama-cli"))
            .args(["--url", &url, "models", "list"])
            .output()
            .expect("execute the CLI against the local API fixture")
    })
    .await
    .expect("CLI task completes");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Modified: 2026-11-18 03:30:00"));
}
