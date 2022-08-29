pub trait BaseServer {
    fn url(&self) -> &'_ str;
}

pub trait BaseClient<TServer: BaseServer + Default> {
    fn new(options: ClientOptions<TServer>) -> Self;
}

pub struct ClientOptions<TServer: BaseServer + Default> {
    pub server: TServer,
}

impl<TServer: BaseServer + Default> Default for ClientOptions<TServer> {
    fn default() -> Self {
        Self {
            server: TServer::default(),
        }
    }
}
