use actix_web::error::InternalError;
use actix_web::HttpResponse;
use reqwest::header::LOCATION;

pub fn e500<T>(e: T) -> InternalError<T> {
    InternalError::from_response(e, HttpResponse::InternalServerError().finish())
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
