// chrono is a time library for Rust
use chrono::Datelike;

use rocket::serde::json::Json;
// import our Date object from the routes/date module
use crate::routes::statue::Status;

pub fn get_current_date() -> Status {
    let current_utc = chrono::Utc::now();
    let year = current_utc.year();
    let month = current_utc.month();
    let day = current_utc.day();
    let current_date = Status { day, month, year };
    current_date
}

pub fn date_plus_month(mut date: Json<Status>) -> Status {
    // create mutable variable new_month and assign the given month + 1 to it
    let mut new_month = date.month + 1;
    // If new_month is over twelve (past December), set it to 1 (January)
    if new_month > 12 {
        new_month = 1;
        date.year = date.year + 1;
    }
    // create a new date object and return it
    let new_date = Status {
        day:   date.day,
        month: new_month,
        year:  date.year,
    };
    new_date
}
