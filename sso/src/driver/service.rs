use crate::{AuditDiff, AuditDiffBuilder, AuditSubject, Core, CoreError, CoreResult};
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use serde_json::Value;
use std::fmt;
use url::Url;
use uuid::Uuid;

/// Service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
    pub is_enabled: bool,
    pub name: String,
    pub url: String,
    pub provider_local_url: Option<String>,
    pub provider_github_oauth2_url: Option<String>,
    pub provider_microsoft_oauth2_url: Option<String>,
}

impl Service {
    /// Check service is enabled.
    pub fn check(self) -> CoreResult<Self> {
        if !self.is_enabled {
            Err(CoreError::ServiceDisabled)
        } else {
            Ok(self)
        }
    }

    /// Build a local provider callback URL with type and serialisable data.
    pub fn provider_local_callback_url<T: Into<String>, D: Serialize>(
        &self,
        type_: T,
        data: D,
    ) -> CoreResult<Url> {
        #[derive(Serialize, Deserialize)]
        struct ServiceCallbackQuery<S: Serialize> {
            #[serde(rename = "type")]
            type_: String,
            #[serde(flatten)]
            data: S,
        }

        let provider_local_url = self
            .provider_local_url
            .as_ref()
            .ok_or_else(|| CoreError::ServiceProviderLocalDisabled)?;
        let mut url = Url::parse(provider_local_url).unwrap();
        let query = ServiceCallbackQuery {
            type_: type_.into(),
            data,
        };
        let query = Core::qs_ser(&query)?;
        url.set_query(Some(&query));
        Ok(url)
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Service {}", self.id)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tupdated_at {}", self.updated_at)?;
        write!(f, "\n\tis_enabled {}", self.is_enabled)?;
        write!(f, "\n\tname {}", self.name)?;
        write!(f, "\n\turl {}", self.url)?;
        if let Some(provider_local_url) = &self.provider_local_url {
            write!(f, "\n\tprovider_local_url {}", provider_local_url)?;
        }
        if let Some(provider_github_oauth2_url) = &self.provider_github_oauth2_url {
            write!(
                f,
                "\n\tprovider_github_oauth2_url {}",
                provider_github_oauth2_url
            )?;
        }
        if let Some(provider_microsoft_oauth2_url) = &self.provider_microsoft_oauth2_url {
            write!(
                f,
                "\n\tprovider_microsoft_oauth2_url {}",
                provider_microsoft_oauth2_url
            )?;
        }
        Ok(())
    }
}

impl AuditSubject for Service {
    fn subject(&self) -> String {
        format!("{}", self.id)
    }
}

impl AuditDiff for Service {
    fn diff(&self, previous: &Self) -> Value {
        let c_provider_local_url = self.provider_local_url.as_ref().map(|x| &**x).unwrap_or("");
        let p_provider_local_url = previous
            .provider_local_url
            .as_ref()
            .map(|x| &**x)
            .unwrap_or("");
        let c_provider_github_oauth2_url = self
            .provider_github_oauth2_url
            .as_ref()
            .map(|x| &**x)
            .unwrap_or("");
        let p_provider_github_oauth2_url = previous
            .provider_github_oauth2_url
            .as_ref()
            .map(|x| &**x)
            .unwrap_or("");
        let c_provider_microsoft_oauth2_url = self
            .provider_microsoft_oauth2_url
            .as_ref()
            .map(|x| &**x)
            .unwrap_or("");
        let p_provider_microsoft_oauth2_url = previous
            .provider_microsoft_oauth2_url
            .as_ref()
            .map(|x| &**x)
            .unwrap_or("");

        AuditDiffBuilder::default()
            .compare("is_enabled", &self.is_enabled, &previous.is_enabled)
            .compare("name", &self.name, &previous.name)
            .compare("url", &self.url, &previous.url)
            .compare(
                "provider_local_url",
                &c_provider_local_url,
                &p_provider_local_url,
            )
            .compare(
                "provider_github_oauth2_url",
                &c_provider_github_oauth2_url,
                &p_provider_github_oauth2_url,
            )
            .compare(
                "provider_microsoft_oauth2_url",
                &c_provider_microsoft_oauth2_url,
                &p_provider_microsoft_oauth2_url,
            )
            .into_value()
    }
}

/// Service list query.
#[derive(Debug)]
pub enum ServiceListQuery {
    Limit(i64),
    IdGt(Uuid, i64),
    IdLt(Uuid, i64),
}

/// Service list filter.
#[derive(Debug)]
pub struct ServiceListFilter {
    pub id: Option<Vec<Uuid>>,
    pub is_enabled: Option<bool>,
}

/// Service list.
#[derive(Debug)]
pub struct ServiceList<'a> {
    pub query: &'a ServiceListQuery,
    pub filter: &'a ServiceListFilter,
}

/// Service create.
#[derive(Debug)]
pub struct ServiceCreate {
    pub is_enabled: bool,
    pub name: String,
    pub url: String,
    pub provider_local_url: Option<String>,
    pub provider_github_oauth2_url: Option<String>,
    pub provider_microsoft_oauth2_url: Option<String>,
}

/// Service update.
#[derive(Debug)]
pub struct ServiceUpdate {
    pub is_enabled: Option<bool>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub provider_local_url: Option<String>,
    pub provider_github_oauth2_url: Option<String>,
    pub provider_microsoft_oauth2_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[derive(Serialize)]
    struct CallbackData {
        email: String,
        token: String,
    }

    #[test]
    fn service_provider_local_callback_url() {
        let id = "6a9c6cfb7e15498b99e057153f0a212b";
        let id = Uuid::parse_str(id).unwrap();
        let service = Service {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            id,
            is_enabled: true,
            name: "Service Name".to_owned(),
            url: "http://localhost:9000".to_owned(),
            provider_local_url: Some("http://localhost:9000".to_owned()),
            provider_github_oauth2_url: None,
            provider_microsoft_oauth2_url: None,
        };
        let callback_data = CallbackData {
            email: "user@test.com".to_owned(),
            token: "6a9c6cfb7e15498b99e057153f0a212b".to_owned(),
        };
        let url = service
            .provider_local_callback_url("reset_password", &callback_data)
            .unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost:9000/?type=reset_password&email=user%40test.com&token=6a9c6cfb7e15498b99e057153f0a212b"
        );
    }
}