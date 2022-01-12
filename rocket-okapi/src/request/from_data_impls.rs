use super::OpenApiFromData;
use crate::gen::OpenApiGenerator;
use okapi::{
    openapi3::{MediaType, RequestBody},
    Map,
};
use rocket::data::Data;
use rocket::serde::json::Json;
use schemars::JsonSchema;
use serde::Deserialize;
use std::{borrow::Cow, result::Result as StdResult};

type Result = crate::Result<RequestBody>;

const DEFAULT_MIME_TYPE: &str = "application/octet-stream";

fn get_mime_type<'a>(mime_type: Option<&'a str>, def: &'static str) -> &'a str
{
    match mime_type {
        Some(t) => t,
        None => def
    }
}

macro_rules! fn_request_body {
    ($gen:ident, $ty:path, $mime_type:expr) => {{
        let schema = $gen.json_schema::<$ty>();
        Ok(RequestBody {
            content: {
                let mut map = Map::new();
                map.insert(
                    $mime_type.to_owned(),
                    MediaType {
                        schema: Some(schema),
                        ..MediaType::default()
                    },
                );
                map
            },
            required: true,
            ..okapi::openapi3::RequestBody::default()
        })
    }};
}

impl<'r> OpenApiFromData<'r> for String {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, String, get_mime_type(mime_type, DEFAULT_MIME_TYPE))
    }
}

impl<'r> OpenApiFromData<'r> for &'r str {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, str, get_mime_type(mime_type, DEFAULT_MIME_TYPE))
    }
}

impl<'r> OpenApiFromData<'r> for Cow<'r, str> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, str, get_mime_type(mime_type, DEFAULT_MIME_TYPE))
    }
}

impl<'r> OpenApiFromData<'r> for Vec<u8> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, Vec<u8>, get_mime_type(mime_type, DEFAULT_MIME_TYPE))
    }
}

impl<'r> OpenApiFromData<'r> for &'r [u8] {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        Vec::<u8>::request_body(gen, mime_type)
    }
}

impl<'r, T: OpenApiFromData<'r> + 'r> OpenApiFromData<'r> for StdResult<T, T::Error> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        T::request_body(gen, mime_type)
    }
}

impl<'r, T: OpenApiFromData<'r>> OpenApiFromData<'r> for Option<T> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        Ok(RequestBody {
            required: false,
            ..T::request_body(gen, mime_type)?
        })
    }
}

// Waiting for https://github.com/GREsau/schemars/issues/103
impl<'r> OpenApiFromData<'r> for rocket::fs::TempFile<'r> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        Vec::<u8>::request_body(gen, mime_type)
    }
}
impl<'r> OpenApiFromData<'r> for rocket::data::Capped<rocket::fs::TempFile<'r>> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        rocket::fs::TempFile::request_body(gen, mime_type)
    }
}
impl<'r> OpenApiFromData<'r> for rocket::data::Capped<Cow<'r, str>> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, str, get_mime_type(mime_type, DEFAULT_MIME_TYPE))
    }
}
impl<'r> OpenApiFromData<'r> for rocket::data::Capped<&'r str> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, str, get_mime_type(mime_type, DEFAULT_MIME_TYPE))
    }
}
// See: https://github.com/GREsau/schemars/issues/103
impl<'r> OpenApiFromData<'r> for rocket::data::Capped<&'r rocket::http::RawStr> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        <&'r rocket::http::RawStr>::request_body(gen, mime_type)
    }
}
impl<'r> OpenApiFromData<'r> for rocket::data::Capped<&'r [u8]> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        Vec::<u8>::request_body(gen, mime_type)
    }
}
impl<'r> OpenApiFromData<'r> for rocket::data::Capped<String> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        String::request_body(gen, mime_type)
    }
}
impl<'r> OpenApiFromData<'r> for rocket::data::Capped<Vec<u8>> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        Vec::<u8>::request_body(gen, mime_type)
    }
}

// See: https://github.com/GREsau/schemars/issues/103
impl<'r> OpenApiFromData<'r> for &'r rocket::http::RawStr {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        Vec::<u8>::request_body(gen, mime_type)
    }
}

impl<'r> OpenApiFromData<'r> for Data<'r> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        Vec::<u8>::request_body(gen, mime_type)
    }
}

// `OpenApiFromForm` is correct, not a mistake, as Rocket requires `FromForm`.
impl<'r, T: JsonSchema + super::OpenApiFromForm<'r>> OpenApiFromData<'r> for rocket::form::Form<T> {
    fn request_body(gen: &mut OpenApiGenerator, mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, T, get_mime_type(mime_type, DEFAULT_MIME_TYPE))
    }
}

impl<'r, T: JsonSchema + Deserialize<'r>> OpenApiFromData<'r> for Json<T> {
    fn request_body(gen: &mut OpenApiGenerator, _mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, T, "application/json")
    }
}

#[cfg(feature = "msgpack")]
impl<'r, T: JsonSchema + Deserialize<'r>> OpenApiFromData<'r>
    for rocket::serde::msgpack::MsgPack<T>
{
    fn request_body(gen: &mut OpenApiGenerator, _mime_type: Option<&str>) -> Result {
        fn_request_body!(gen, T, "application/msgpack")
    }
}
