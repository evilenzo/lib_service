use opentelemetry::trace::TraceContextExt;
use opentelemetry::{trace::TracerProvider as _, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{self, RandomIdGenerator},
    Resource,
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing::span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_tracing(service_name: &str) {
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://jaeger:4317"),
        )
        .with_trace_config(
            trace::Config::default()
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![KeyValue::new(
                    SERVICE_NAME,
                    service_name.to_string(),
                )])),
        )
        .install_batch(runtime::Tokio)
        .expect("Failed to initialize tracer provider");

    let tracer = tracer_provider.tracer("websocket-communicator");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", service_name).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default tracing subscriber");

    // TODO: Idk now if it is need or not
    // opentelemetry::global::set_tracer_provider(tracer_provider.clone());
}

#[allow(dead_code)]
pub trait WithInfoPropagation {
    fn with_propagation(&self) -> &Self;
}

impl WithInfoPropagation for span::Span {
    fn with_propagation(&self) -> &Self {
        self.set_attribute(
            "meta.trace_id",
            format!("{:032x}", self.context().span().span_context().trace_id()),
        );
        self.set_attribute(
            "meta.span_id",
            format!("{:016x}", self.context().span().span_context().span_id()),
        );
        self
    }
}
