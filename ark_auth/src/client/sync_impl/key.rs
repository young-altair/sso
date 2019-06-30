use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{KeyCreateBody, KeyReadResponse};

impl SyncClient {
    pub fn key_create(
        &self,
        is_enabled: bool,
        name: &str,
        service_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<KeyReadResponse, Error> {
        let body = KeyCreateBody {
            is_enabled,
            name: name.to_owned(),
            service_id: service_id.map(|x| x.to_owned()),
            user_id: user_id.map(|x| x.to_owned()),
        };

        self.post_json("/v1/key", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(|_err| Error::Unwrap))
    }
}
