// auto_api::gen_api!(petstore_api, "https://petstore.swagger.io/v2/swagger.json");
// auto_api::gen_api!(petstore_staging_api, "https://petstore.swagger.io/v2/swagger.json", "http://localhost:8080");
// auto_api::gen_api!(exchange_rate_api, "https://api.apis.guru/v2/specs/exchangerate-api.com/4/openapi.json");

#[auto_api::gen_api("https://api.apis.guru/v2/specs/weatherbit.io/2.0.0/swagger.json")]
pub mod petstore_api {}

#[auto_api::gen_api("file://example/openapi/petstore.json")]
pub mod petstore_api_local {}

fn main() {
    let petstore_client = petstore_api_local::Client::default();
    let base_url = &petstore_client.base_url;
    println!("Base URL: {base_url}");
    //
}
