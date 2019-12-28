//! gRPC server and clients.
mod client;
mod http;
mod methods;
mod options;
mod util;

pub mod pb {
    //! Generated protobuf server and client items.
    tonic::include_proto!("sso");
}

pub use crate::grpc::pb::sso_client::SsoClient as Client;
pub use crate::grpc::{client::*, http::*, options::*};

use crate::*;
use lettre::{file::FileTransport, SmtpClient, Transport};
use lettre_email::Email;
use prometheus::{HistogramTimer, HistogramVec, IntCounterVec};
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// gRPC server.
#[derive(Clone)]
pub struct Server {
    options: ServerOptions,
    driver: Arc<Box<dyn Driver>>,
    client: Arc<reqwest::Client>,
    smtp_client: Arc<Option<SmtpClient>>,
    count: IntCounterVec,
    latency: HistogramVec,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server {{ driver }}")
    }
}

impl Server {
    /// Returns a new `Server`.
    pub fn new(driver: Box<dyn Driver>, options: ServerOptions) -> Self {
        let client = options.client().unwrap();
        let smtp_client = options.smtp_client().unwrap();
        let (count, latency) = Metrics::http_metrics();
        Self {
            options,
            driver: Arc::new(driver),
            client: Arc::new(client),
            smtp_client: Arc::new(smtp_client),
            count,
            latency,
        }
    }

    /// Returns reference to driver.
    pub fn driver(&self) -> Arc<Box<dyn Driver>> {
        self.driver.clone()
    }

    /// Build email callback function. Must be called from blocking context.
    /// If client is None and file directory path is provided, file transport is used.
    pub fn smtp_email(&self) -> Box<dyn FnOnce(TemplateEmail) -> DriverResult<()>> {
        let client = self.smtp_client.clone();
        let from_email = self.options.smtp_from_email();
        let smtp_file = self.options.smtp_file();

        Box::new(move |email| {
            let email_builder = Email::builder()
                .to((email.to_email, email.to_name))
                .subject(email.subject)
                .text(email.text);

            match (client.as_ref(), smtp_file) {
                (Some(client), _) => {
                    let email = email_builder
                        .from((from_email.unwrap(), email.from_name))
                        .build()
                        .map_err(DriverError::LettreEmail)?;

                    let mut transport = client.clone().transport();
                    transport.send(email.into()).map_err(DriverError::Lettre)?;
                    Ok(())
                }
                (_, Some(smtp_file)) => {
                    let email = email_builder
                        .from(("file@localhost", email.from_name))
                        .build()
                        .map_err(DriverError::LettreEmail)?;

                    let path = PathBuf::from(smtp_file);
                    let mut transport = FileTransport::new(path);
                    transport
                        .send(email.into())
                        .map_err(DriverError::LettreFile)?;
                    Ok(())
                }
                (None, None) => Err(DriverError::SmtpDisabled),
            }
        })
    }

    pub fn metrics_start(&self, path: &str) -> (HistogramTimer, IntCounterVec) {
        let timer = self.latency.with_label_values(&[path]).start_timer();
        (timer, self.count.clone())
    }

    pub fn metrics_end(
        &self,
        timer: HistogramTimer,
        count: IntCounterVec,
        path: &str,
        status: &str,
    ) {
        timer.observe_duration();
        count.with_label_values(&[path, status]).inc_by(1);
    }
}

#[tonic::async_trait]
impl pb::sso_server::Sso for Server {
    async fn ping(&self, _: Request<()>) -> Result<Response<String>, Status> {
        Err(Status::not_found(""))
    }

    async fn metrics(&self, _: Request<()>) -> Result<Response<String>, Status> {
        Err(Status::not_found(""))
    }

    async fn audit_list(
        &self,
        request: Request<pb::AuditListRequest>,
    ) -> Result<Response<pb::AuditListReply>, Status> {
        methods::audit::list(self.driver.clone(), request).await
    }

    async fn audit_create(
        &self,
        request: Request<pb::AuditCreateRequest>,
    ) -> Result<Response<pb::AuditReadReply>, Status> {
        methods::audit::create(self.driver.clone(), request).await
    }

    async fn audit_read(
        &self,
        request: Request<pb::AuditReadRequest>,
    ) -> Result<Response<pb::AuditReadReply>, Status> {
        methods::audit::read(self.driver.clone(), request).await
    }

    async fn audit_update(
        &self,
        request: Request<pb::AuditUpdateRequest>,
    ) -> Result<Response<pb::AuditReadReply>, Status> {
        methods::audit::update(self.driver.clone(), request).await
    }

    async fn key_list(
        &self,
        request: tonic::Request<pb::KeyListRequest>,
    ) -> Result<tonic::Response<pb::KeyListReply>, tonic::Status> {
        methods::key::list(self.driver.clone(), request).await
    }

    async fn key_create(
        &self,
        request: tonic::Request<pb::KeyCreateRequest>,
    ) -> Result<tonic::Response<pb::KeyCreateReply>, tonic::Status> {
        methods::key::create(self.driver.clone(), request).await
    }

    async fn key_read(
        &self,
        request: tonic::Request<pb::KeyReadRequest>,
    ) -> Result<tonic::Response<pb::KeyReadReply>, tonic::Status> {
        methods::key::read(self.driver.clone(), request).await
    }

    async fn key_update(
        &self,
        request: tonic::Request<pb::KeyUpdateRequest>,
    ) -> Result<tonic::Response<pb::KeyReadReply>, tonic::Status> {
        methods::key::update(self.driver.clone(), request).await
    }

    async fn key_delete(
        &self,
        request: tonic::Request<pb::KeyReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        methods::key::delete(self.driver.clone(), request).await
    }

    async fn service_list(
        &self,
        request: tonic::Request<pb::ServiceListRequest>,
    ) -> Result<tonic::Response<pb::ServiceListReply>, tonic::Status> {
        methods::service::list(self.driver.clone(), request).await
    }

    async fn service_create(
        &self,
        request: tonic::Request<pb::ServiceCreateRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        methods::service::create(self.driver.clone(), request).await
    }

    async fn service_read(
        &self,
        request: tonic::Request<pb::ServiceReadRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        methods::service::read(self.driver.clone(), request).await
    }

    async fn service_update(
        &self,
        request: tonic::Request<pb::ServiceUpdateRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        methods::service::update(self.driver.clone(), request).await
    }

    async fn service_delete(
        &self,
        request: tonic::Request<pb::ServiceReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        methods::service::delete(self.driver.clone(), request).await
    }

    async fn user_list(
        &self,
        request: tonic::Request<pb::UserListRequest>,
    ) -> Result<tonic::Response<pb::UserListReply>, tonic::Status> {
        methods::user::list(self.driver.clone(), request).await
    }

    async fn user_create(
        &self,
        request: tonic::Request<pb::UserCreateRequest>,
    ) -> Result<tonic::Response<pb::UserReadReply>, tonic::Status> {
        methods::user::create(self.driver.clone(), request).await
    }

    async fn user_read(
        &self,
        request: tonic::Request<pb::UserReadRequest>,
    ) -> Result<tonic::Response<pb::UserReadReply>, tonic::Status> {
        methods::user::read(self.driver.clone(), request).await
    }

    async fn user_update(
        &self,
        request: tonic::Request<pb::UserUpdateRequest>,
    ) -> Result<tonic::Response<pb::UserReadReply>, tonic::Status> {
        methods::user::update(self.driver.clone(), request).await
    }

    async fn user_delete(
        &self,
        request: tonic::Request<pb::UserReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        methods::user::delete(self.driver.clone(), request).await
    }
}
