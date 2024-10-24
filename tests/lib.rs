use compute_logger::DatadogLog;
use log::{Level, Log, Metadata, Record};
use mockall::{mock, predicate};
use serde_json::Value;
use std::fmt;

mock! {
    pub Log {}

    impl log::Log for Log {
        fn enabled<'a>(&self, metadata: &Metadata<'a>) -> bool;

        fn log<'a>(&self, record: &Record<'a>);

        fn flush(&self);
    }
}

#[test]
fn enabled_calls_nested_log_enabled() {
    let mut nested = MockLog::new();

    let metadata = Metadata::builder()
        .level(Level::Debug)
        .target("target")
        .build();

    let expected_metadata = metadata.clone();

    nested
        .expect_enabled()
        .withf(move |m| *m == expected_metadata)
        .times(1)
        .return_const(true);

    let datadog_log = DatadogLog::new("SOURCE", "TAGS", "HOSTNAME", "SERVICE", nested);

    assert!(datadog_log.enabled(&metadata));
}

#[test]
fn flush_calls_nested_log_flush() {
    let mut nested = MockLog::new();
    nested.expect_flush().times(1).return_const(());

    let datadog_log = DatadogLog::new("SOURCE", "TAGS", "HOSTNAME", "SERVICE", nested);

    datadog_log.flush()
}

#[test]
fn log_calls_nested_log_with_metadata() {
    let mut nested = MockLog::new();

    nested
        .expect_enabled()
        .with(predicate::always())
        .return_const(true);

    nested
        .expect_log()
        .withf(move |r| {
            let record_matches = r.level() == Level::Debug
                && r.target() == "SomeTarget"
                && r.file() == Some("file.rs")
                && r.line() == Some(123)
                && r.module_path() == Some("module_path");

            let m = serde_json::from_str::<serde_json::Value>(&fmt::format(*r.args()))
                .expect("Parsed JSON");

            let json_matches = m["ddsource"] == "SOURCE"
                && m["ddtags"] == "TAGS"
                && m["hostname"] == "HOSTNAME"
                && m["service"] == "SERVICE";

            let message_matches = match &m["message"] {
                Value::String(message) => message.ends_with(" DEBUG Some args"),
                _ => false,
            };

            record_matches && json_matches && message_matches
        })
        .times(1)
        .return_const(());

    let datadog_log = DatadogLog::new("SOURCE", "TAGS", "HOSTNAME", "SERVICE", nested);

    datadog_log.log(
        &Record::builder()
            .args(format_args!("Some {}", "args"))
            .level(Level::Debug)
            .target("SomeTarget")
            .file(Some("file.rs"))
            .line(Some(123))
            .module_path(Some("module_path"))
            .build(),
    );
}

#[test]
fn log_does_not_call_nested_log_when_not_enabled() {
    let mut nested = MockLog::new();

    nested.expect_log().never();

    nested
        .expect_enabled()
        .with(predicate::always())
        .return_const(false);

    let datadog_log = DatadogLog::new("SOURCE", "TAGS", "HOSTNAME", "SERVICE", nested);

    datadog_log.log(
        &Record::builder()
            .args(format_args!("Some {}", "args"))
            .level(Level::Debug)
            .target("SomeTarget")
            .file(Some("file.rs"))
            .line(Some(123))
            .module_path(Some("module_path"))
            .build(),
    );
}
