use std::sync::Arc;
use toolbox::foundationdb::Database;

pub struct State {
    tenant: Option<String>,
    database: Arc<Database>,
    authenticated: bool,
}

impl State {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            tenant: None,
            database,
            authenticated: false,
        }
    }
    pub fn tenant(&self) -> Option<&str> {
        self.tenant.as_deref()
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn set_tenant(&mut self, tenant: &str) {
        self.tenant = Some(tenant.to_string());
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
    
    pub fn set_authenticated(&mut self, authenticated: bool) {
        self.authenticated = authenticated;
    }
}
