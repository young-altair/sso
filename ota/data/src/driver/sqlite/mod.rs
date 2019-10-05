//! # SQLite Driver
mod model;
mod schema;

use crate::core::{Data, Disk, DiskOptions, DiskStatus, Key, KeyStatus, Status, Version};
use crate::driver::{Driver, Error};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use std::convert::TryInto;

embed_migrations!("migrations/sqlite");

#[derive(Clone)]
pub struct SqliteDriver {
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

impl SqliteDriver {
    pub fn initialise(database_url: &str, max_connections: Option<u32>) -> Result<Self, Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let mut pool = r2d2::Pool::builder();
        if let Some(max_connections) = max_connections {
            pool = pool.max_size(max_connections);
        }
        let pool = pool.build(manager).map_err(Error::R2d2)?;
        let driver = SqliteDriver { pool };

        let connection = driver.connection()?;
        embedded_migrations::run(&connection).map_err(Error::DieselMigrations)?;
        connection
            .execute("PRAGMA foreign_keys = ON")
            .map_err(Error::Diesel)?;

        Ok(driver)
    }

    fn connection(&self) -> Result<PooledConnection, Error> {
        self.pool.get().map_err(Error::R2d2)
    }

    fn uuid() -> String {
        uuid::Uuid::new_v4().to_simple().to_string()
    }

    fn disk_list_where_name_gte_inner(
        &self,
        name_gte: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<String>, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        kv_disk
            .select(disk_id)
            .filter(disk_name.ge(name_gte))
            .limit(limit)
            .offset(offset)
            .order(disk_name.asc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }

    fn key_list_where_name_gte_inner(
        &self,
        name_gte: &str,
        limit: i64,
        offset: i64,
        key_disk_id: &str,
    ) -> Result<Vec<String>, Error> {
        use crate::driver::sqlite::schema::kv_key::dsl::*;

        let conn = self.connection()?;
        kv_key
            .select(key_id)
            .filter(key_name.ge(name_gte).and(disk_id.eq(key_disk_id)))
            .limit(limit)
            .offset(offset)
            .order(key_name.asc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }

    fn version_list_where_created_lte_inner(
        &self,
        created_lte: &str,
        limit: i64,
        offset: i64,
        version_key_id: &str,
    ) -> Result<Vec<String>, Error> {
        use crate::driver::sqlite::schema::kv_version::dsl::*;

        let conn = self.connection()?;
        kv_version
            .select(version_id)
            .filter(created_at.le(created_lte).and(key_id.eq(version_key_id)))
            .limit(limit)
            .offset(offset)
            .order(created_at.desc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }

    fn data_delete_by_version_id(&self, id: &str) -> Result<usize, Error> {
        use crate::driver::sqlite::schema::kv_data::dsl::*;

        let conn = self.connection()?;
        diesel::delete(kv_data.filter(version_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }
}

impl Driver for SqliteDriver {
    fn box_clone(&self) -> Box<dyn Driver> {
        Box::new((*self).clone())
    }

    fn status(&self) -> Result<Status, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        let disk_count = kv_disk
            .select(diesel::dsl::count_star())
            .get_result::<i64>(&conn)
            .map_err(Error::Diesel)?;

        Ok(Status { disk_count })
    }

    fn vacuum(&self) -> Result<usize, Error> {
        let conn = self.connection()?;
        conn.execute("VACUUM").map_err(Error::Diesel)
    }

    fn disk_list_where_name_gte(
        &self,
        name_gte: &str,
        offset_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<String>, Error> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        self.disk_list_where_name_gte_inner(name_gte, limit, offset)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, id) in res.iter().enumerate() {
                        if id == offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return self.disk_list_where_name_gte_inner(name_gte, limit, offset);
                        }
                    }
                }
                Ok(res)
            })
    }

    fn disk_create(&self, name: &str, options: &DiskOptions) -> Result<Disk, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        let now = Utc::now().to_rfc3339();
        let id = SqliteDriver::uuid();
        let options = serde_json::to_string(options).unwrap();
        let value = model::DiskInsert {
            created_at: &now,
            updated_at: &now,
            disk_id: &id,
            disk_name: name,
            disk_options: &options,
        };
        diesel::insert_into(kv_disk)
            .values(&value)
            .execute(&conn)
            .map_err(Error::Diesel)?;
        let disk = self.disk_read_by_id(&id)?;
        Ok(disk.unwrap())
    }

    fn disk_status_by_id(&self, id: &str) -> Result<DiskStatus, Error> {
        use crate::driver::sqlite::schema::kv_key::dsl::*;

        let disk = self.disk_read_by_id(id)?.unwrap();
        let conn = self.connection()?;
        let key_count = kv_key
            .filter(disk_id.eq(id))
            .select(diesel::dsl::count_star())
            .get_result::<i64>(&conn)
            .map_err(Error::Diesel)?;
        // TODO(refactor): Implement this.
        let total_size = 0;

        Ok(DiskStatus {
            id: disk.id,
            name: disk.name,
            key_count,
            total_size,
        })
    }

    fn disk_read_by_id(&self, id: &str) -> Result<Option<Disk>, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        kv_disk
            .filter(disk_id.eq(id))
            .get_result::<model::Disk>(&conn)
            .map(|x| Some(x.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn disk_read_by_name(&self, name: &str) -> Result<Option<Disk>, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        kv_disk
            .filter(disk_name.eq(name))
            .get_result::<model::Disk>(&conn)
            .map(|x| Some(x.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn disk_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        // TODO(refactor): Use scan in place of large limits.
        let key_list = self.key_list_where_name_gte("", None, 65536, id)?;
        for key in key_list {
            self.key_delete_by_id(&key)?;
        }

        let conn = self.connection()?;
        diesel::delete(kv_disk.filter(disk_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn key_list_where_name_gte(
        &self,
        name_gte: &str,
        offset_id: Option<&str>,
        limit: i64,
        disk_id: &str,
    ) -> Result<Vec<String>, Error> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        self.key_list_where_name_gte_inner(name_gte, limit, offset, disk_id)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, id) in res.iter().enumerate() {
                        if id == offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return self
                                .key_list_where_name_gte_inner(name_gte, limit, offset, disk_id);
                        }
                    }
                }
                Ok(res)
            })
    }

    fn key_create(&self, name: &str, key_disk_id: &str) -> Result<Key, Error> {
        use crate::driver::sqlite::schema::kv_key::dsl::*;

        let conn = self.connection()?;
        let now = Utc::now().to_rfc3339();
        let id = SqliteDriver::uuid();
        let value = model::KeyInsert {
            created_at: &now,
            updated_at: &now,
            key_id: &id,
            key_name: name,
            disk_id: key_disk_id,
        };
        diesel::insert_into(kv_key)
            .values(&value)
            .execute(&conn)
            .map_err(Error::Diesel)?;
        let key = self.key_read_by_id(&id)?;
        Ok(key.unwrap())
    }

    fn key_status_by_id(&self, id: &str) -> Result<KeyStatus, Error> {
        use crate::driver::sqlite::schema::kv_version::dsl::*;

        let key = self.key_read_by_id(id)?.unwrap();
        let conn = self.connection()?;
        let version_count = kv_version
            .filter(key_id.eq(id))
            .select(diesel::dsl::count_star())
            .get_result::<i64>(&conn)
            .map_err(Error::Diesel)?;
        // TODO(refactor): Implement this.
        let total_size = 0;

        Ok(KeyStatus {
            id: key.id,
            name: key.name,
            version_count,
            total_size,
        })
    }

    fn key_read_by_id(&self, id: &str) -> Result<Option<Key>, Error> {
        use crate::driver::sqlite::schema::kv_key::dsl::*;

        let conn = self.connection()?;
        kv_key
            .filter(key_id.eq(id))
            .get_result::<model::Key>(&conn)
            .map(|x| Some(x.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_read_by_name(&self, name: &str, key_disk_id: &str) -> Result<Option<Key>, Error> {
        use crate::driver::sqlite::schema::kv_key::dsl::*;

        let conn = self.connection()?;
        kv_key
            .filter(key_name.eq(name).and(disk_id.eq(key_disk_id)))
            .get_result::<model::Key>(&conn)
            .map(|x| Some(x.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_update_by_id(
        &self,
        id: &str,
        name: Option<&str>,
        key_version_id: Option<&str>,
    ) -> Result<usize, Error> {
        use crate::driver::sqlite::schema::kv_key::dsl::*;

        let conn = self.connection()?;
        let now = chrono::Utc::now().to_rfc3339();
        let value = model::KeyUpdate {
            updated_at: &now,
            key_name: name,
            version_id: key_version_id,
        };
        diesel::update(kv_key.filter(key_id.eq(id)))
            .set(&value)
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn key_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        use crate::driver::sqlite::schema::kv_key::dsl::*;

        let now = Utc::now();
        let version_list = self.version_list_where_created_lte(&now, None, 1024, id)?;
        for version in version_list {
            self.version_delete_by_id(&version)?;
        }

        let conn = self.connection()?;
        diesel::delete(kv_key.filter(key_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn version_list_where_created_lte(
        &self,
        created_lte: &DateTime<Utc>,
        offset_id: Option<&str>,
        limit: i64,
        key_id: &str,
    ) -> Result<Vec<String>, Error> {
        let created_lte = created_lte.to_rfc3339();
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        self.version_list_where_created_lte_inner(&created_lte, limit, offset, key_id)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, id) in res.iter().enumerate() {
                        if id == offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return self.version_list_where_created_lte_inner(
                                &created_lte,
                                limit,
                                offset,
                                key_id,
                            );
                        }
                    }
                }
                Ok(res)
            })
    }

    fn version_create(
        &self,
        hash: &[u8],
        size: i64,
        version_key_id: &str,
    ) -> Result<Version, Error> {
        use crate::driver::sqlite::schema::kv_version::dsl::*;

        let conn = self.connection()?;
        let now = Utc::now().to_rfc3339();
        let id = SqliteDriver::uuid();
        let value = model::VersionInsert {
            created_at: &now,
            version_id: &id,
            version_hash: hash,
            version_size: size,
            key_id: version_key_id,
        };
        diesel::insert_into(kv_version)
            .values(&value)
            .execute(&conn)
            .map_err(Error::Diesel)?;
        let version = self.version_read_by_id(&id)?;
        Ok(version.unwrap())
    }

    fn version_read_by_id(&self, id: &str) -> Result<Option<Version>, Error> {
        use crate::driver::sqlite::schema::kv_version::dsl::*;

        let conn = self.connection()?;
        kv_version
            .filter(version_id.eq(id))
            .get_result::<model::Version>(&conn)
            .map(|x| Some(x.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn version_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        use crate::driver::sqlite::schema::kv_version::dsl::*;

        self.data_delete_by_version_id(id)?;
        let conn = self.connection()?;
        diesel::delete(kv_version.filter(version_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn data_create(&self, chunk: i64, data: &[u8], data_version_id: &str) -> Result<Data, Error> {
        use crate::driver::sqlite::schema::kv_data::dsl::*;

        let conn = self.connection()?;
        let value = model::DataInsert {
            data_chunk: chunk,
            data_value: data,
            version_id: data_version_id,
        };
        diesel::insert_into(kv_data)
            .values(&value)
            .execute(&conn)
            .map_err(Error::Diesel)?;
        let data = self.data_read_by_chunk(chunk, data_version_id)?;
        Ok(data.unwrap())
    }

    fn data_read_by_chunk(&self, chunk: i64, data_version_id: &str) -> Result<Option<Data>, Error> {
        use crate::driver::sqlite::schema::kv_data::dsl::*;

        let conn = self.connection()?;
        kv_data
            .filter(data_chunk.eq(chunk).and(version_id.eq(data_version_id)))
            .get_result::<model::Data>(&conn)
            .map(|x| Some(x.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }
}