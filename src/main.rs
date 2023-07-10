#[macro_use] extern crate rocket;
use serde::Deserialize;

#[get("/hello")]
fn hello() -> &'static str {
    "Hello world\n"
}

#[get("/greet?<name>")] // Name'e göre printliyor
fn greet(name: &str) -> String {
    format!("Hello {}\n", name)
}


#[derive(Debug, Deserialize)]
struct GreetingRequest {
    name: String,
}

#[post("/greetj", data = "<request>")] //JSONu alıyor, GreetingRequest'e dönüştürüyor, sonra ismi basıyor
fn greetj(request: &str) -> String {
    let g: GreetingRequest = serde_json::from_str(request).unwrap();
    format!("Hello {}", g.name)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, greet, greetj])
}