use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn cmd() -> Command {
    cargo_bin_cmd!("regulator-cli")
}

// -- Help & subcommand discovery --

#[test]
fn help_shows_all_subcommands() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("new-compliance-definition")
                .and(predicate::str::contains("init"))
                .and(predicate::str::contains("publish"))
                .and(predicate::str::contains("update")),
        );
}

// -- File validation --

#[test]
fn new_compliance_definition_rejects_missing_file() {
    cmd()
        .args(["new-compliance-definition", "nonexistent.nr"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("file not found: nonexistent.nr"));
}

#[test]
fn new_compliance_definition_rejects_wrong_extension() {
    let tmp = tempfile::Builder::new()
        .suffix(".txt")
        .tempfile()
        .unwrap();

    cmd()
        .args(["new-compliance-definition", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("expected a .nr file, got .txt"));
}

#[test]
fn new_compliance_definition_rejects_no_extension() {
    let dir = tempfile::tempdir().unwrap();
    let no_ext = dir.path().join("circuit");
    std::fs::write(&no_ext, "fn main() {}").unwrap();

    cmd()
        .args(["new-compliance-definition", no_ext.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("file has no extension, expected .nr"));
}

// -- Stub commands --

#[test]
fn init_is_not_yet_implemented() {
    cmd()
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn publish_is_not_yet_implemented() {
    cmd()
        .arg("publish")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn update_is_not_yet_implemented() {
    cmd()
        .arg("update")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not yet implemented"));
}

// -- IPFS upload (mocked) --

#[tokio::test]
async fn new_compliance_definition_uploads_to_ipfs_and_prints_cid() {
    let mock_server = MockServer::start().await;

    let fake_cid = "QmTestCid1234567890abcdef";

    Mock::given(method("POST"))
        .and(path("/api/v0/add"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Name": "main.nr",
            "Hash": fake_cid,
            "Size": "42"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create a temp .nr file
    let dir = tempfile::tempdir().unwrap();
    let circuit_file = dir.path().join("main.nr");
    {
        let mut f = std::fs::File::create(&circuit_file).unwrap();
        writeln!(f, "fn main(x: u64, y: pub u64) {{ assert(x != y); }}").unwrap();
    }

    cmd()
        .args([
            "--ipfs-rpc-url",
            &mock_server.uri(),
            "new-compliance-definition",
            circuit_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(fake_cid));
}

#[tokio::test]
async fn new_compliance_definition_reports_ipfs_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v0/add"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let circuit_file = dir.path().join("test.nr");
    std::fs::write(&circuit_file, "fn main() {}").unwrap();

    cmd()
        .args([
            "--ipfs-rpc-url",
            &mock_server.uri(),
            "new-compliance-definition",
            circuit_file.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("IPFS add failed"));
}

#[tokio::test]
async fn ipfs_rpc_url_env_var_is_used() {
    let mock_server = MockServer::start().await;

    let fake_cid = "QmEnvVarTestCid";

    Mock::given(method("POST"))
        .and(path("/api/v0/add"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Name": "circuit.nr",
            "Hash": fake_cid,
            "Size": "10"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let circuit_file = dir.path().join("circuit.nr");
    std::fs::write(&circuit_file, "fn main() {}").unwrap();

    cmd()
        .env("IPFS_RPC_URL", &mock_server.uri())
        .args([
            "new-compliance-definition",
            circuit_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(fake_cid));
}
