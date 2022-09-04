#[auto_api::gen_api("file://openapi/petstore.json")]
pub mod petstore_api {
    pub fn world() -> &'static str {
        "Hello World!"
    }
}

fn main() {
    println!("{}", petstore_api::world());
    // let petstore_client = petstore_api::Client::default();
    // let base_url = &petstore_client.base_url;
    // println!("Base URL: {base_url}");
}
