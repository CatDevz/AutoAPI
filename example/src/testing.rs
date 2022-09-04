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

mod petstore_api {
    use auto_api::__private::reqwest;

    use super::{BaseClient, BaseServer, ClientOptions};

    //// Settings
    pub enum Server {
        Production,
        Staging,
        Custom { url: String },
    }

    impl BaseServer for Server {
        fn url(&self) -> &str {
            match self {
                Server::Production => "https://example.com",
                Server::Staging => "https://beta.example.com",
                Server::Custom { url } => url,
            }
        }
    }

    impl Default for Server {
        fn default() -> Self {
            Self::Production
        }
    }

    //// Definitions
    pub struct Category {
        pub id: i64,
        pub name: String,
    }

    pub struct Pet {
        pub id: Option<i64>,
        pub category: Option<Category>,
        pub name: String,
        pub photo_urls: Vec<String>,
        pub tags: Option<Vec<Tag>>,
        pub status: Option<PetStatus>,
    }

    pub enum PetStatus {
        Available,
        Pending,
        Sold,
    }

    pub struct Tag {
        pub id: i64,
        pub name: String,
    }

    //// Request Builders
    #[derive(Clone)]
    pub struct GetPetByIdRequest<'a> {
        client: &'a Client,
        pet_id: i64,
        owner_id: Option<i64>,
    }

    impl GetPetByIdRequest<'_> {
        pub async fn send(self) -> Result<Pet, ()> {
            Err(())
        }

        pub fn send_blocking(self) -> Result<Pet, ()> {
            Err(())
        }
    }

    impl GetPetByIdRequest<'_> {
        pub fn pet_id(mut self, pet_id: i64) -> Self {
            self.pet_id = pet_id;
            self
        }

        pub fn owner_id(mut self, owner_id: Option<i64>) -> Self {
            self.owner_id = owner_id;
            self
        }
    }

    //// Client
    pub struct Client {
        inner: reqwest::Client,
        pub base_url: String,
    }

    impl Client {
        pub fn get_pet_by_id(&self, pet_id: i64) -> GetPetByIdRequest {
            GetPetByIdRequest {
                client: &self,
                pet_id,
                owner_id: None,
            }
        }
    }

    impl BaseClient<Server> for Client {
        fn new(options: ClientOptions<Server>) -> Self {
            Self {
                inner: reqwest::Client::new(),
                base_url: options.server.url().to_string(),
            }
        }
    }

    impl Default for Client {
        fn default() -> Self {
            Self::new(Default::default())
        }
    }
}

async fn _usage() -> Result<(), ()> {
    let client = petstore_api::Client::new(ClientOptions {
        server: petstore_api::Server::Production,
        ..Default::default()
    });

    let response = client.get_pet_by_id(32).owner_id(Some(52)).send().await?;
    let _name = response.name;

    Ok(())
}
