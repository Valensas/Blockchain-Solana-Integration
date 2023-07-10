#[macro_use] extern crate rocket;
use serde::{Deserialize, Serialize};
/*use rocket::config::{Config, Environment};

let config = Config::build(Environment::Staging)
    .port(9234)
    .finalize()?;



    rocket::custom(config)
    .mount("/", routes![/* .. */])
    .launch();
*/

#[get("/hello")]
fn hello() -> &'static str {
    "Hello world\n"
}

/*#[get("/greet?<name>")] //TODO: ? ne işe yarıyor
fn greet(name: &str) -> String {
    format!("Hello {}\n", name)
}*/

/*#[post("/greet")]
fn greet(name: &str) -> String {
    format!("Hello {}\n", name)
}*/

#[derive(Debug, Deserialize)]
struct GreetingRequest {
    name: String,
}

#[post("/greet", data = "<request>")]
fn greet(request: Json<GreetingRequest>) -> String {
    format!("Hello {}", request.name)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, greet])
}
