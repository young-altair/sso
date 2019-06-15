use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::route::service::{
    CreateBody, CreateResponse, ListQuery, ListResponse, ReadResponse,
};
use actix_web::http::StatusCode;

impl SyncClient {
    pub fn service_list(
        &self,
        gt: Option<i64>,
        lt: Option<i64>,
        limit: Option<i64>,
    ) -> Result<ListResponse, Error> {
        let query = ListQuery { gt, lt, limit };

        self.get_query("/v1/service", query)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(res),
                _ => Err(Error::Unwrap),
            })
            .and_then(|mut res| res.json::<ListResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn service_create(&self, name: &str, url: &str) -> Result<CreateResponse, Error> {
        let body = CreateBody {
            name: name.to_owned(),
            url: url.to_owned(),
        };

        self.post_json("/v1/service", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(res),
                _ => Err(Error::Unwrap),
            })
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn service_read(&self, id: i64) -> Result<ReadResponse, Error> {
        let path = format!("/v1/service/{}", id);

        self.get(&path)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(res),
                _ => Err(Error::Unwrap),
            })
            .and_then(|mut res| res.json::<ReadResponse>().map_err(|_err| Error::Unwrap))
    }
}