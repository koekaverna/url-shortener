#[macro_use] extern crate rocket;
use rand::Rng;
use rand::distributions::Alphanumeric;
use rocket::request::FromRequest;
use rocket::response::Redirect;
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
use redis;
use redis::Commands;
use redis::RedisResult;

const URL: &str = "http://127.0.0.1:8000";
const REDIS_URL: &str = "redis://redis:6379";

#[get("/")]
fn index() -> Template {
    Template::render("shortener", context! {
        url: "",
        short_url: ""
    })
}

#[get("/?<url>")]
fn shortener(url: &str) -> Template {
    let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let short_url = format!("{}/{}", URL, id);

    save_url(&id, url);

    Template::render("shortener", context! {
        url,
        short_url
    })
}

struct Url(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Url {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let id: &str = request.uri().path().as_str().trim_start_matches('/');
        match get_url(id) {
            Ok(url) => rocket::request::Outcome::Success(Url(url)),
            Err(_) => rocket::request::Outcome::Forward(())
        }
    }
}

#[get("/<_id>")]
fn url(_id: &str, url: Url) -> Redirect {
    Redirect::to(url.0)
}

#[catch(404)]
fn not_found() -> Template {
    Template::render("404", context! {})
}

fn save_url(id: &str, url: &str) {
    let client = redis::Client::open(REDIS_URL).unwrap();
    let mut con = client.get_connection().unwrap();
    let _: () = con.set(id, url).unwrap();
}

fn get_url(id: &str) -> RedisResult<String> {
    let client = redis::Client::open(REDIS_URL)?;
    let mut con = client.get_connection()?;

    con.get(id)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, shortener, url])
        .attach(Template::fairing())
        .register("/", catchers![not_found])
}
