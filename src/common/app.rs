use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use tokio::signal;

pub struct App {
    router: Option<Router<()>>,
}

pub trait AppBuilder {
    fn to_app(self) -> App;
}

impl AppBuilder for Router<()> {
    fn to_app(self) -> App {
        App {
            router: Some(
                self.layer(OtelInResponseLayer::default())
                    .layer(OtelAxumLayer::default()),
            ),
        }
    }
}

impl App {
    #[allow(dead_code)]
    pub async fn run(&mut self) {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        axum::serve(listener, self.router.take().expect("App was not built!"))
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .unwrap();
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        opentelemetry::global::shutdown_tracer_provider();
    }
}
