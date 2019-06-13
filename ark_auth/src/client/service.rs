use crate::client::{Client, ClientError};
use crate::server::route::service::{
    CreateBody, CreateResponse, ListQuery, ListResponse, ReadResponse,
};
use actix_web::http::StatusCode;
use futures::{future, Future};

impl Client {
    pub fn service_list(
        &self,
        gt: Option<i64>,
        lt: Option<i64>,
        limit: Option<i64>,
    ) -> impl Future<Item = ListResponse, Error = ClientError> {
        let query = ListQuery { gt, lt, limit };

        self.get_query("/v1/service", query)
            .send()
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<ListResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }

    pub fn service_create(
        &self,
        name: &str,
        url: &str,
    ) -> impl Future<Item = CreateResponse, Error = ClientError> {
        let body = CreateBody {
            name: name.to_owned(),
            url: url.to_owned(),
        };

        self.post("/v1/service")
            .send_json(&body)
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<CreateResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }

    pub fn service_read(&self, id: i64) -> impl Future<Item = ReadResponse, Error = ClientError> {
        let path = format!("/v1/service/{}", id);

        self.get(&path)
            .send()
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<ReadResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }
}
