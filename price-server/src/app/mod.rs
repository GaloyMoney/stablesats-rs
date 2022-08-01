pub struct App {}

impl App {
    pub fn run() -> Self {
        let subscriber = Subscriber::new(config).await?;
        Self {}
    }
}
