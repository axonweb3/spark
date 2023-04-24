// import Rocket
#[macro_use]
extern crate rocket;

mod routes;
mod services;

// import our routes
use routes::statue::date_plus_month;
use routes::statue::get_current_date;

#[launch]
fn main() -> () {
    rocket::build().mount("/api", routes![get_current_date, date_plus_month])
}
