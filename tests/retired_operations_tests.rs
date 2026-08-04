//! Retired SDK methods fail locally instead of sending fictional HTTP routes.

#![allow(deprecated)]

use redis_enterprise::debuginfo::DebugInfoHandler;
use redis_enterprise::{
    ActionHandler, AlertHandler, ClusterHandler, CmSettingsHandler, CrdbTasksHandler,
    CreateCrdbTaskRequest, CreateSuffixRequest, DiagnosticsHandler, EndpointsHandler,
    EnterpriseClient, JobSchedulerHandler, LicenseHandler, MigrationsHandler, ModuleHandler,
    OcspHandler, ProxyHandler, RestError, RolesHandler, SuffixesHandler,
};
use serde_json::json;

macro_rules! assert_unsupported {
    ($future:expr) => {
        assert!(matches!(
            $future.await,
            Err(RestError::UnsupportedOperation(_))
        ));
    };
}

fn client() -> EnterpriseClient {
    EnterpriseClient::builder()
        .base_url("http://127.0.0.1:1")
        .username("admin")
        .password("password")
        .build()
        .unwrap()
}

#[tokio::test]
async fn retired_operations_return_the_explicit_local_error() {
    let client = client();

    assert_unsupported!(ActionHandler::new(client.clone()).cancel("action"));
    assert_unsupported!(AlertHandler::new(client.clone()).get_settings("alert"));
    assert_unsupported!(ClusterHandler::new(client.clone()).settings());
    assert_unsupported!(CmSettingsHandler::new(client.clone()).reset());
    assert_unsupported!(
        CrdbTasksHandler::new(client.clone()).create(
            CreateCrdbTaskRequest::builder()
                .crdb_guid("crdb")
                .task_type("sync")
                .build()
        )
    );
    assert_unsupported!(DebugInfoHandler::new(client.clone()).list());
    assert_unsupported!(DiagnosticsHandler::new(client.clone()).list_reports());
    assert_unsupported!(EndpointsHandler::new(client.clone()).list());
    assert_unsupported!(JobSchedulerHandler::new(client.clone()).list());
    assert_unsupported!(LicenseHandler::new(client.clone()).usage());
    assert_unsupported!(MigrationsHandler::new(client.clone()).list());
    assert_unsupported!(ModuleHandler::new(client.clone()).update("module", json!({})));
    assert_unsupported!(OcspHandler::new(client.clone()).query());
    assert_unsupported!(ProxyHandler::new(client.clone()).stats(1));
    assert_unsupported!(RolesHandler::new(client.clone()).built_in());
    assert_unsupported!(
        SuffixesHandler::new(client).create(
            CreateSuffixRequest::builder()
                .name("suffix")
                .dns_suffix("example.com")
                .build()
        )
    );
}
