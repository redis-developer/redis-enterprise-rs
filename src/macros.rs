//! Internal macros for reducing handler boilerplate
//!
//! These macros generate standard CRUD handler implementations.

/// Defines a handler struct with a `client` field and `new()` constructor.
///
/// # Example
///
/// ```ignore
/// define_handler!(
///     /// Documentation for the handler
///     pub struct MyHandler;
/// );
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! define_handler {
    (
        $(#[$meta:meta])*
        $vis:vis struct $handler:ident;
    ) => {
        $(#[$meta])*
        $vis struct $handler {
            client: $crate::client::RestClient,
        }

        impl $handler {
            /// Creates a new handler with the given client.
            pub fn new(client: $crate::client::RestClient) -> Self {
                Self { client }
            }
        }
    };
}

/// Implements standard CRUD methods for a handler.
///
/// Supports the following method specifications:
/// - `list => Entity, "/path"` - generates `list() -> Result<Vec<Entity>>`
/// - `get(u32) => Entity, "/path/{}"` - generates `get(uid: u32) -> Result<Entity>`
/// - `get(&str) => Entity, "/path/{}"` - generates `get(uid: &str) -> Result<Entity>`
/// - `delete(u32), "/path/{}"` - generates `delete(uid: u32) -> Result<()>`
/// - `delete(&str), "/path/{}"` - generates `delete(uid: &str) -> Result<()>`
/// - `create(Request) => Entity, "/path"` - generates `create(request: Request) -> Result<Entity>`
/// - `update(u32, Request) => Entity, "/path/{}"` - generates `update(uid: u32, request: Request) -> Result<Entity>`
/// - `update(&str, Request) => Entity, "/path/{}"` - generates `update(uid: &str, request: Request) -> Result<Entity>`
///
/// # Example
///
/// ```ignore
/// impl_crud!(MyHandler {
///     list => MyEntity, "/v1/resources";
///     get(u32) => MyEntity, "/v1/resources/{}";
///     delete(u32), "/v1/resources/{}";
///     create(CreateRequest) => MyEntity, "/v1/resources";
///     update(u32, UpdateRequest) => MyEntity, "/v1/resources/{}";
/// });
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! impl_crud {
    // Entry point - parse method specifications
    ($handler:ty { $($methods:tt)* }) => {
        impl $handler {
            $crate::impl_crud!(@methods $($methods)*);
        }
    };

    // List method
    (@methods list => $entity:ty, $path:literal; $($rest:tt)*) => {
        /// List all resources
        pub async fn list(&self) -> $crate::error::Result<Vec<$entity>> {
            self.client.get($path).await
        }
        $crate::impl_crud!(@methods $($rest)*);
    };

    // Get method with u32 ID
    (@methods get(u32) => $entity:ty, $path:literal; $($rest:tt)*) => {
        /// Get a specific resource by UID
        pub async fn get(&self, uid: u32) -> $crate::error::Result<$entity> {
            self.client.get(&format!($path, uid)).await
        }
        $crate::impl_crud!(@methods $($rest)*);
    };

    // Get method with &str ID
    (@methods get(&str) => $entity:ty, $path:literal; $($rest:tt)*) => {
        /// Get a specific resource by ID
        pub async fn get(&self, uid: &str) -> $crate::error::Result<$entity> {
            self.client.get(&format!($path, uid)).await
        }
        $crate::impl_crud!(@methods $($rest)*);
    };

    // Delete method with u32 ID
    (@methods delete(u32), $path:literal; $($rest:tt)*) => {
        /// Delete a resource by UID
        pub async fn delete(&self, uid: u32) -> $crate::error::Result<()> {
            self.client.delete(&format!($path, uid)).await
        }
        $crate::impl_crud!(@methods $($rest)*);
    };

    // Delete method with &str ID
    (@methods delete(&str), $path:literal; $($rest:tt)*) => {
        /// Delete a resource by ID
        pub async fn delete(&self, uid: &str) -> $crate::error::Result<()> {
            self.client.delete(&format!($path, uid)).await
        }
        $crate::impl_crud!(@methods $($rest)*);
    };

    // Create method
    (@methods create($req:ty) => $entity:ty, $path:literal; $($rest:tt)*) => {
        /// Create a new resource
        pub async fn create(&self, request: $req) -> $crate::error::Result<$entity> {
            self.client.post($path, &request).await
        }
        $crate::impl_crud!(@methods $($rest)*);
    };

    // Update method with u32 ID
    (@methods update(u32, $req:ty) => $entity:ty, $path:literal; $($rest:tt)*) => {
        /// Update an existing resource
        pub async fn update(&self, uid: u32, request: $req) -> $crate::error::Result<$entity> {
            self.client.put(&format!($path, uid), &request).await
        }
        $crate::impl_crud!(@methods $($rest)*);
    };

    // Update method with &str ID
    (@methods update(&str, $req:ty) => $entity:ty, $path:literal; $($rest:tt)*) => {
        /// Update an existing resource
        pub async fn update(&self, uid: &str, request: $req) -> $crate::error::Result<$entity> {
            self.client.put(&format!($path, uid), &request).await
        }
        $crate::impl_crud!(@methods $($rest)*);
    };

    // Base case - no more methods
    (@methods) => {};
}

#[cfg(test)]
mod tests {
    // Test that macros compile correctly
    // All items are intentionally unused - we're just verifying compilation
    #[allow(dead_code)]
    mod test_handler {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct TestEntity {
            pub uid: u32,
            pub name: String,
        }

        #[derive(Debug, Serialize)]
        pub struct CreateTestRequest {
            pub name: String,
        }

        define_handler!(
            /// Test handler for macro validation
            pub struct TestHandler;
        );

        impl_crud!(TestHandler {
            list => TestEntity, "/v1/test";
            get(u32) => TestEntity, "/v1/test/{}";
            delete(u32), "/v1/test/{}";
            create(CreateTestRequest) => TestEntity, "/v1/test";
            update(u32, CreateTestRequest) => TestEntity, "/v1/test/{}";
        });

        // Custom method can still be added
        impl TestHandler {
            /// Custom method example
            pub async fn custom(&self) -> crate::error::Result<Vec<TestEntity>> {
                self.client.get("/v1/test/custom").await
            }
        }
    }

    #[test]
    fn test_macro_compiles() {
        // If this compiles, the macros work
        // Actual functionality is tested via wiremock in handler tests
    }
}
