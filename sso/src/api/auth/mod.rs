mod github;
mod local;
mod microsoft;

pub use crate::api::auth::{github::*, local::*, microsoft::*};

use crate::{
    api::{
        result_audit, result_audit_err, validate, ApiResult, AuditCreate2Request,
        AuditIdOptResponse, ValidateRequest, ValidateRequestQuery,
    },
    AuditBuilder, AuditMeta, AuditType, Csrf, Driver, UserKey, UserToken, UserTokenAccess,
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenRequest {
    #[validate(custom = "validate::token")]
    pub token: String,
    pub audit: Option<AuditCreate2Request>,
}

impl ValidateRequest<AuthTokenRequest> for AuthTokenRequest {}

impl AuthTokenRequest {
    pub fn new<S1: Into<String>>(token: S1, audit: Option<AuditCreate2Request>) -> Self {
        Self {
            token: token.into(),
            audit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenResponse {
    pub data: UserToken,
    pub audit: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenAccessResponse {
    pub data: UserTokenAccess,
    pub audit: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthKeyRequest {
    #[validate(custom = "validate::key")]
    pub key: String,
    pub audit: Option<AuditCreate2Request>,
}

impl ValidateRequest<AuthKeyRequest> for AuthKeyRequest {}

impl AuthKeyRequest {
    pub fn new<S: Into<String>>(key: S, audit: Option<AuditCreate2Request>) -> Self {
        Self {
            key: key.into(),
            audit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthKeyResponse {
    pub data: UserKey,
    pub audit: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTotpRequest {
    pub user_id: Uuid,
    #[validate(custom = "validate::totp")]
    pub totp: String,
}

impl ValidateRequest<AuthTotpRequest> for AuthTotpRequest {}

impl AuthTotpRequest {
    pub fn new<S: Into<String>>(user_id: Uuid, totp: S) -> Self {
        Self {
            user_id,
            totp: totp.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct AuthCsrfCreateRequest {
    #[validate(custom = "validate::csrf_expires_s")]
    pub expires_s: Option<i64>,
}

impl ValidateRequest<AuthCsrfCreateRequest> for AuthCsrfCreateRequest {}
impl ValidateRequestQuery<AuthCsrfCreateRequest> for AuthCsrfCreateRequest {}

impl AuthCsrfCreateRequest {
    pub fn new(expires_s: i64) -> Self {
        Self {
            expires_s: Some(expires_s),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthCsrfCreateResponse {
    pub data: Csrf,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthCsrfVerifyRequest {
    #[validate(custom = "validate::csrf_key")]
    pub key: String,
}

impl ValidateRequest<AuthCsrfVerifyRequest> for AuthCsrfVerifyRequest {}

impl AuthCsrfVerifyRequest {
    pub fn new<S: Into<String>>(key: S) -> Self {
        Self { key: key.into() }
    }
}

/// Authentication provider OAuth2 options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProviderOauth2 {
    pub client_id: String,
    pub client_secret: String,
}

impl AuthProviderOauth2 {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }
}

/// Authentication provider OAuth2 common arguments.
#[derive(Debug)]
pub struct AuthProviderOauth2Args<'a> {
    provider: Option<&'a AuthProviderOauth2>,
    user_agent: String,
    access_token_expires: i64,
    refresh_token_expires: i64,
}

impl<'a> AuthProviderOauth2Args<'a> {
    pub fn new<S1: Into<String>>(
        provider: Option<&'a AuthProviderOauth2>,
        user_agent: S1,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> Self {
        Self {
            provider,
            user_agent: user_agent.into(),
            access_token_expires,
            refresh_token_expires,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthOauth2UrlResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthOauth2CallbackRequest {
    #[validate(custom = "validate::token")]
    pub code: String,
    #[validate(custom = "validate::token")]
    pub state: String,
}

impl ValidateRequest<AuthOauth2CallbackRequest> for AuthOauth2CallbackRequest {}
impl ValidateRequestQuery<AuthOauth2CallbackRequest> for AuthOauth2CallbackRequest {}

pub fn auth_key_verify(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthKeyRequest,
) -> ApiResult<AuthKeyResponse> {
    AuthKeyRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthKeyVerify);

    let res = server_auth::key_verify(driver, &mut audit, key_value, request);
    result_audit_err(driver, &audit, res).map(|(data, audit)| AuthKeyResponse {
        data,
        audit: audit.map(|x| x.id),
    })
}

pub fn auth_key_revoke(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthKeyRequest,
) -> ApiResult<AuditIdOptResponse> {
    AuthKeyRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthKeyRevoke);

    let res = server_auth::key_revoke(driver, &mut audit, key_value, request);
    result_audit(driver, &audit, res).map(|audit| AuditIdOptResponse {
        audit: audit.map(|x| x.id),
    })
}

pub fn auth_token_verify(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthTokenRequest,
) -> ApiResult<AuthTokenAccessResponse> {
    AuthTokenRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthTokenVerify);

    let res = server_auth::token_verify(driver, &mut audit, key_value, request);
    result_audit_err(driver, &audit, res).map(|(data, audit)| AuthTokenAccessResponse {
        data,
        audit: audit.map(|x| x.id),
    })
}

pub fn auth_token_refresh(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthTokenRequest,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> ApiResult<AuthTokenResponse> {
    AuthTokenRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthTokenRefresh);

    let res = server_auth::token_refresh(
        driver,
        &mut audit,
        key_value,
        request,
        access_token_expires,
        refresh_token_expires,
    );
    result_audit_err(driver, &audit, res).map(|(data, audit)| AuthTokenResponse {
        data,
        audit: audit.map(|x| x.id),
    })
}

pub fn auth_token_revoke(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthTokenRequest,
) -> ApiResult<AuditIdOptResponse> {
    AuthTokenRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthTokenRevoke);

    let res = server_auth::token_revoke(driver, &mut audit, key_value, request);
    result_audit(driver, &audit, res).map(|audit| AuditIdOptResponse {
        audit: audit.map(|x| x.id),
    })
}

pub fn auth_totp(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthTotpRequest,
) -> ApiResult<()> {
    AuthTotpRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthTotp);

    let res = server_auth::totp(driver, &mut audit, key_value, request);
    result_audit_err(driver, &audit, res)
}

pub fn auth_csrf_create(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthCsrfCreateRequest,
    csrf_token_expires: i64,
) -> ApiResult<AuthCsrfCreateResponse> {
    AuthCsrfCreateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthCsrfCreate);

    let res = server_auth::csrf_create(driver, &mut audit, key_value, request, csrf_token_expires);
    result_audit_err(driver, &audit, res).map(|data| AuthCsrfCreateResponse { data })
}

pub fn auth_csrf_verify(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthCsrfVerifyRequest,
) -> ApiResult<()> {
    AuthCsrfVerifyRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthCsrfVerify);

    let res = server_auth::csrf_verify(driver, &mut audit, key_value, request);
    result_audit_err(driver, &audit, res)
}

mod server_auth {
    use super::*;
    use crate::{
        api::{ApiError, ApiResult},
        Audit, AuditBuilder, Auth, CoreError, Csrf, Driver, Jwt, Key, KeyType, Service, UserToken,
        UserTokenAccess,
    };

    pub fn key_verify(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthKeyRequest,
    ) -> ApiResult<(UserKey, Option<Audit>)> {
        let service =
            Auth::authenticate_service(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Key verify requires key key type.
        let key = Auth::key_read_by_user_value(driver, &service, audit, request.key, KeyType::Key)
            .map_err(ApiError::BadRequest)?;
        let user = Auth::user_read_by_id(driver, Some(&service), audit, key.user_id.unwrap())
            .map_err(ApiError::BadRequest)?;

        // Key verified.
        let user_key = UserKey {
            user,
            key: key.value,
        };

        // Optionally create custom audit log.
        if let Some(x) = request.audit {
            let audit = audit
                .create(driver, x.into())
                .map_err(ApiError::BadRequest)?;
            Ok((user_key, Some(audit)))
        } else {
            Ok((user_key, None))
        }
    }

    pub fn key_revoke(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthKeyRequest,
    ) -> ApiResult<Option<Audit>> {
        let service =
            Auth::authenticate_service(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Key revoke requires key key type.
        // Do not check key is enabled or not revoked.
        let key = Auth::key_read_by_user_value_unchecked(
            driver,
            &service,
            audit,
            request.key,
            KeyType::Key,
        )
        .map_err(ApiError::BadRequest)?;

        // Disable and revoke key.
        Key::update(
            driver,
            Some(&service),
            key.id,
            Some(false),
            Some(true),
            None,
        )
        .map_err(ApiError::BadRequest)?;

        // Optionally create custom audit log.
        if let Some(x) = request.audit {
            let audit = audit
                .create(driver, x.into())
                .map_err(ApiError::BadRequest)?;
            Ok(Some(audit))
        } else {
            Ok(None)
        }
    }

    pub fn token_verify(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthTokenRequest,
    ) -> ApiResult<(UserTokenAccess, Option<Audit>)> {
        let service =
            Auth::authenticate_service(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) =
            Jwt::decode_unsafe(&request.token, service.id).map_err(ApiError::BadRequest)?;

        // Token verify requires token key type.
        let user = Auth::user_read_by_id(driver, Some(&service), audit, user_id)
            .map_err(ApiError::BadRequest)?;
        let key = Auth::key_read_by_user(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Safely decode token with user key.
        let access_token_expires = Auth::decode_access_token(&service, &user, &key, &request.token)
            .map_err(ApiError::BadRequest)?;

        // Token verified.
        let user_token = UserTokenAccess {
            user,
            access_token: request.token,
            access_token_expires,
        };

        // Optionally create custom audit log.
        if let Some(x) = request.audit {
            let audit = audit
                .create(driver, x.into())
                .map_err(ApiError::BadRequest)?;
            Ok((user_token, Some(audit)))
        } else {
            Ok((user_token, None))
        }
    }

    pub fn token_refresh(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthTokenRequest,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> ApiResult<(UserToken, Option<Audit>)> {
        let service =
            Auth::authenticate_service(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) =
            Jwt::decode_unsafe(&request.token, service.id).map_err(ApiError::BadRequest)?;

        // Token refresh requires token key type.
        let user = Auth::user_read_by_id(driver, Some(&service), audit, user_id)
            .map_err(ApiError::BadRequest)?;
        let key = Auth::key_read_by_user(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Safely decode token with user key.
        let csrf_key = Auth::decode_refresh_token(&service, &user, &key, &request.token)
            .map_err(ApiError::BadRequest)?;

        // Verify CSRF to prevent reuse.
        Auth::csrf_verify(driver, &service, csrf_key).map_err(ApiError::BadRequest)?;

        // Encode user token.
        let user_token = Auth::encode_user_token(
            driver,
            &service,
            user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )
        .map_err(ApiError::BadRequest)?;

        // Optionally create custom audit log.
        if let Some(x) = request.audit {
            let audit = audit
                .create(driver, x.into())
                .map_err(ApiError::BadRequest)?;
            Ok((user_token, Some(audit)))
        } else {
            Ok((user_token, None))
        }
    }

    pub fn token_revoke(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthTokenRequest,
    ) -> ApiResult<Option<Audit>> {
        let service =
            Auth::authenticate_service(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, token_type) =
            Jwt::decode_unsafe(&request.token, service.id).map_err(ApiError::BadRequest)?;

        // Token revoke requires token key type.
        // Do not check user, key is enabled or not revoked.
        let user = Auth::user_read_by_id_unchecked(driver, Some(&service), audit, user_id)
            .map_err(ApiError::BadRequest)?;
        let key = Auth::key_read_by_user_unchecked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Safely decode token with user key.
        let csrf_key = Auth::decode_csrf_key(&service, &user, &key, token_type, &request.token)
            .map_err(ApiError::BadRequest)?;
        if let Some(csrf_key) = csrf_key {
            Csrf::read_opt(driver, csrf_key).map_err(ApiError::BadRequest)?;
        }

        // Token revoked, disable and revoked linked key.
        Key::update(
            driver,
            Some(&service),
            key.id,
            Some(false),
            Some(true),
            None,
        )
        .map_err(ApiError::BadRequest)?;

        // Optionally create custom audit log.
        if let Some(x) = request.audit {
            let audit = audit
                .create(driver, x.into())
                .map_err(ApiError::BadRequest)?;
            Ok(Some(audit))
        } else {
            Ok(None)
        }
    }

    pub fn totp(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthTotpRequest,
    ) -> ApiResult<()> {
        let service =
            Auth::authenticate_service(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // TOTP requires token key type.
        let user = Auth::user_read_by_id(driver, Some(&service), audit, request.user_id)
            .map_err(ApiError::BadRequest)?;
        let key = Auth::key_read_by_user(driver, &service, audit, &user, KeyType::Totp)
            .map_err(ApiError::BadRequest)?;

        // Verify TOTP code.
        Auth::totp(&key.value, &request.totp).map_err(ApiError::BadRequest)
    }

    pub fn csrf_create(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthCsrfCreateRequest,
        csrf_token_expires: i64,
    ) -> ApiResult<Csrf> {
        let service =
            Auth::authenticate_service(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        let expires_s = request.expires_s.unwrap_or(csrf_token_expires);
        Auth::csrf_create(driver, &service, expires_s).map_err(ApiError::BadRequest)
    }

    pub fn csrf_verify(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthCsrfVerifyRequest,
    ) -> ApiResult<()> {
        let service =
            Auth::authenticate_service(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        Auth::csrf_verify(driver, &service, request.key).map_err(ApiError::BadRequest)
    }

    pub fn oauth2_login(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        service: &Service,
        service_id: Uuid,
        email: String,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> ApiResult<UserToken> {
        // Check service making url and callback requests match.
        if service.id != service_id {
            return Err(ApiError::BadRequest(CoreError::CsrfServiceMismatch));
        }

        // OAuth2 login requires token key type.
        let user = Auth::user_read_by_email(driver, Some(&service), audit, email)
            .map_err(ApiError::BadRequest)?;
        let key = Auth::key_read_by_user(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Encode user token.
        Auth::encode_user_token(
            driver,
            &service,
            user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )
        .map_err(ApiError::BadRequest)
    }
}