use chrono::prelude::{SecondsFormat, Utc};
use log;
use std::fmt;

pub struct DatadogLog<L: log::Log> {
    source: String,
    tags: String,
    hostname: String,
    service: String,
    nested: L,
}

impl<L: log::Log> DatadogLog<L> {
    pub fn new(
        source: &str,
        tags: &str,
        hostname: &str,
        service: &str,
        nested: L,
    ) -> DatadogLog<L> {
        DatadogLog {
            source: source.to_string(),
            tags: tags.to_string(),
            hostname: hostname.to_string(),
            service: service.to_string(),
            nested,
        }
    }
}

impl<L: log::Log> log::Log for DatadogLog<L> {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.nested.enabled(metadata)
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = format!(
            "{timestamp} {level} {message}",
            timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            level = record.level(),
            message = fmt::format(*record.args())
        );

        println!("{}", message);

        self.nested.log(
            &log::Record::builder()
                .metadata(record.metadata().clone())
                .args(format_args!(
                    "{}",
                    serde_json::json!({
                        "ddsource": self.source,
                        "ddtags": self.tags,
                        "hostname": self.hostname,
                        "message": message,
                        "service": self.service
                    })
                ))
                .line(record.line())
                .file(record.file())
                .module_path(record.module_path())
                .build(),
        )
    }

    fn flush(&self) {
        self.nested.flush()
    }
}
