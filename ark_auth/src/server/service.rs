use crate::{
    core,
    server::{
        route_json_config, route_response_empty, route_response_json, validate_name,
        validate_unsigned, Data, Error, ValidateFromValue,
    },
};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::Future;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct ListQuery {
    #[validate(custom = "validate_unsigned")]
    gt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    lt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    limit: Option<i64>,
}

impl ValidateFromValue<ListQuery> for ListQuery {}

#[derive(Debug, Serialize, Deserialize)]
struct ListMetaResponse {
    gt: Option<i64>,
    lt: Option<i64>,
    limit: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    meta: ListMetaResponse,
    data: Vec<core::Service>,
}

fn list_handler(
    data: web::Data<Data>,
    id: Identity,
    query: web::Query<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    ListQuery::from_value(query.into_inner())
        .and_then(|query| {
            web::block(move || list_inner(data.get_ref(), id, &query)).map_err(Into::into)
        })
        .then(|res| route_response_json(res))
}

fn list_inner(data: &Data, id: Option<String>, query: &ListQuery) -> Result<ListResponse, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| {
            let limit = query.limit.unwrap_or(10);
            let (gt, lt, services) = match query.lt {
                Some(lt) => {
                    let services =
                        core::service::list_where_id_lt(data.driver(), &service, lt, limit)?;
                    (None, Some(lt), services)
                }
                None => {
                    let gt = query.gt.unwrap_or(0);
                    let services =
                        core::service::list_where_id_gt(data.driver(), &service, gt, limit)?;
                    (Some(gt), None, services)
                }
            };

            Ok(ListResponse {
                meta: ListMetaResponse { gt, lt, limit },
                data: services,
            })
        })
        .map_err(Into::into)
}

// TODO(feature): Create new services via API.
// #[derive(Debug, Serialize, Deserialize, Validate)]
// #[serde(deny_unknown_fields)]
// struct CreateBody {
//     #[validate(custom = "validate_name")]
//     name: String,
//     #[validate(url)]
//     url: String,
// }
// impl ValidateFromValue<CreateBody> for CreateBody {}

#[derive(Debug, Serialize, Deserialize)]
struct ReadResponse {
    data: core::Service,
}

fn read_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    web::block(move || read_inner(data.get_ref(), id, path.0))
        .map_err(Into::into)
        .then(|res| route_response_json(res))
}

fn read_inner(data: &Data, id: Option<String>, service_id: i64) -> Result<ReadResponse, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| core::service::read_by_id(data.driver(), &service, service_id))
        .map_err(Into::into)
        .and_then(|service| service.ok_or_else(|| Error::NotFound))
        .map(|service| ReadResponse { data: service })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct UpdateBody {
    #[validate(custom = "validate_name")]
    name: Option<String>,
}

impl ValidateFromValue<UpdateBody> for UpdateBody {}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateResponse {
    data: core::Service,
}

fn update_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    UpdateBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || update_inner(data.get_ref(), id, path.0, &body)).map_err(Into::into)
        })
        .then(|res| route_response_json(res))
}

fn update_inner(
    data: &Data,
    id: Option<String>,
    service_id: i64,
    body: &UpdateBody,
) -> Result<UpdateResponse, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| {
            core::service::update_by_id(
                data.driver(),
                &service,
                service_id,
                body.name.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
        .map(|service| UpdateResponse { data: service })
}

fn delete_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    web::block(move || delete_inner(data.get_ref(), id, path.0))
        .map_err(Into::into)
        .then(|res| route_response_empty(res))
}

fn delete_inner(data: &Data, id: Option<String>, service_id: i64) -> Result<usize, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| core::service::delete_by_id(data.driver(), &service, service_id))
        .map_err(Into::into)
}

/// API version 1 service scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/service")
        .service(
            web::resource("")
                .route(web::get().to_async(list_handler))
                // TODO(feature): Create new services via API, root keys?
                // .route(
                //     web::post()
                //         .data(route_json_config())
                //         .to_async(create_handler),
                // ),
        )
        .service(
            web::resource("/{service_id}")
                .route(web::get().to_async(read_handler))
                .route(
                    web::patch()
                        .data(route_json_config())
                        .to_async(update_handler),
                )
                .route(web::delete().to_async(delete_handler)),
        )
}