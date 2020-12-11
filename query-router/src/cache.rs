use crate::error::Error;
use lru_cache::LruCache;
use schema_registry::types::SchemaType;
use std::sync::{Mutex, MutexGuard};
use utils::abort_on_poison;
use uuid::Uuid;

pub struct AddressCache {
    schema_registry_addr: String,
    addresses: Mutex<LruCache<Uuid, String>>,
}

impl AddressCache {
    pub fn new(schema_registry_addr: String, capacity: usize) -> Self {
        Self {
            schema_registry_addr,
            addresses: Mutex::new(LruCache::new(capacity)),
        }
    }

    fn lock(&self) -> MutexGuard<LruCache<Uuid, String>> {
        self.addresses.lock().unwrap_or_else(abort_on_poison)
    }

    pub async fn get_address(&self, schema_id: Uuid) -> Result<String, Error> {
        if let Some(address) = self.lock().get_mut(&schema_id) {
            return Ok(address.clone());
        }

        let mut conn = schema_registry::connect_to_registry(self.schema_registry_addr.clone())
            .await
            .map_err(Error::RegistryConnectionError)?;
        let response = conn
            .get_schema_query_address(schema_registry::rpc::schema::Id {
                id: schema_id.to_string(),
            })
            .await
            .map_err(Error::RegistryError)?;
        let address = response.into_inner().address;

        self.lock().insert(schema_id, address.clone());

        Ok(address)
    }
}

pub struct SchemaTypeCache {
    schema_registry_addr: String,
    schema_types: Mutex<LruCache<Uuid, SchemaType>>,
}

impl SchemaTypeCache {
    pub fn new(schema_registry_addr: String, capacity: usize) -> Self {
        Self {
            schema_registry_addr,
            schema_types: Mutex::new(LruCache::new(capacity)),
        }
    }

    fn lock(&self) -> MutexGuard<LruCache<Uuid, SchemaType>> {
        self.schema_types.lock().unwrap_or_else(abort_on_poison)
    }

    pub async fn get_schema_type(&self, schema_id: Uuid) -> Result<SchemaType, Error> {
        if let Some(schema_type) = self.lock().get_mut(&schema_id) {
            return Ok(schema_type.clone());
        }

        let mut conn = schema_registry::connect_to_registry(self.schema_registry_addr.clone())
            .await
            .map_err(Error::RegistryConnectionError)?;
        let response = conn
            .get_schema_type(schema_registry::rpc::schema::Id {
                id: schema_id.to_string(),
            })
            .await
            .map_err(Error::RegistryError)?;
        let schema_type = response.into_inner().schema_type();

        self.lock().insert(schema_id, schema_type.into());

        Ok(schema_type.into())
    }
}
