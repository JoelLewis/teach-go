fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gosensei_app=info,warn".into()),
        )
        .init();
    gosensei_app::run();
}
