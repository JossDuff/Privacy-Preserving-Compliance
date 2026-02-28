use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::path::{Path, PathBuf};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn cmd() -> Command {
    cargo_bin_cmd!("regulator-cli")
}

/// Create a minimal valid Nargo project in a temp directory.
/// Returns (tempdir, project_path) -- keep tempdir alive for the test duration.
fn create_nargo_project(parent: &Path, name: &str, circuit_src: &str) -> PathBuf {
    let project_dir = parent.join(name);
    std::fs::create_dir_all(project_dir.join("src")).unwrap();
    std::fs::write(
        project_dir.join("Nargo.toml"),
        format!(
            "[package]\nname = \"{name}\"\ntype = \"bin\"\nauthors = [\"test\"]\n\n[dependencies]\n"
        ),
    )
    .unwrap();
    std::fs::write(project_dir.join("src/main.nr"), circuit_src).unwrap();
    project_dir
}

fn mock_ipfs_add(fake_cid: &str, file_name: &str, size: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "Name": file_name,
        "Hash": fake_cid,
        "Size": size
    }))
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

// -- Input validation --

#[test]
fn new_compliance_definition_rejects_nonexistent_directory() {
    cmd()
        .args(["new-compliance-definition", "/tmp/nonexistent-noir-project"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn new_compliance_definition_rejects_directory_without_nargo_toml() {
    let dir = tempfile::tempdir().unwrap();

    cmd()
        .args(["new-compliance-definition", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no Nargo.toml found"));
}

// -- Nargo compilation --

#[test]
fn new_compliance_definition_rejects_invalid_circuit() {
    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(dir.path(), "bad_circuit", "this is not valid noir");

    cmd()
        .args(["new-compliance-definition", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("circuit validation failed"));
}

// -- Init command --

#[test]
fn init_creates_noir_project() {
    let dir = tempfile::tempdir().unwrap();

    cmd()
        .current_dir(dir.path())
        .args(["init", "my_circuit"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created compliance definition project: my_circuit/"));

    let project_dir = dir.path().join("my_circuit");
    assert!(project_dir.join("Nargo.toml").exists());
    assert!(project_dir.join("src/main.nr").exists());

    let nargo_toml = std::fs::read_to_string(project_dir.join("Nargo.toml")).unwrap();
    assert!(nargo_toml.contains("name = \"my_circuit\""));
    assert!(nargo_toml.contains("type = \"bin\""));

    let main_nr = std::fs::read_to_string(project_dir.join("src/main.nr")).unwrap();
    assert!(main_nr.contains("fn main"));
}

#[test]
fn init_rejects_existing_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("existing")).unwrap();

    cmd()
        .current_dir(dir.path())
        .args(["init", "existing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("directory already exists"));
}

// -- Publish command --

#[test]
fn publish_requires_path_argument() {
    cmd()
        .arg("publish")
        .assert()
        .failure()
        .stderr(predicate::str::contains("DIR"));
}

#[test]
fn publish_rejects_nonexistent_directory() {
    cmd()
        .args(["publish", "/tmp/nonexistent-noir-project"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn publish_rejects_directory_without_nargo_toml() {
    let dir = tempfile::tempdir().unwrap();

    cmd()
        .args(["publish", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no Nargo.toml found"));
}

#[test]
fn publish_rejects_invalid_circuit() {
    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(dir.path(), "bad_circuit", "this is not valid noir");

    cmd()
        .args(["publish", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nargo compile failed"));
}

#[test]
fn publish_compiles_and_generates_verifier() {
    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(
        dir.path(),
        "test_circuit",
        "fn main(x: u64, y: pub u64) { assert(x != y); }",
    );

    cmd()
        .current_dir(dir.path())
        .args(["publish", project.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Verifier.sol"))
        .stderr(
            predicate::str::contains("circuit compiled successfully")
                .and(predicate::str::contains("verification key generated"))
                .and(predicate::str::contains("Solidity verifier generated")),
        );

    // Default verifier location
    assert!(project.join("target/Verifier.sol").exists());

    // Default receipt should be written
    let receipt_path = dir.path().join("receipt.json");
    assert!(receipt_path.exists());

    let receipt: Value =
        serde_json::from_str(&std::fs::read_to_string(&receipt_path).unwrap()).unwrap();
    assert_eq!(receipt["command"], "publish");
    assert!(receipt["data"]["verifier_path"]
        .as_str()
        .unwrap()
        .ends_with("Verifier.sol"));
    assert!(receipt["data"]["vk_path"]
        .as_str()
        .unwrap()
        .ends_with("vk"));
    assert!(receipt["data"]["bytecode_path"]
        .as_str()
        .unwrap()
        .ends_with(".json"));
}

#[test]
fn publish_verifier_output_flag_overrides_default() {
    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(
        dir.path(),
        "test_circuit",
        "fn main(x: u64, y: pub u64) { assert(x != y); }",
    );
    let custom_verifier = dir.path().join("my-verifier.sol");

    cmd()
        .current_dir(dir.path())
        .args([
            "publish",
            "--verifier-output",
            custom_verifier.to_str().unwrap(),
            project.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-verifier.sol"));

    assert!(custom_verifier.exists());
    // Default location should NOT exist
    assert!(!project.join("target/Verifier.sol").exists());
}

// -- Stub commands --

#[test]
fn update_is_not_yet_implemented() {
    cmd()
        .arg("update")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not yet implemented"));
}

// -- IPFS upload (mocked) with nargo compilation --

#[tokio::test]
async fn new_compliance_definition_compiles_and_uploads() {
    let mock_server = MockServer::start().await;
    let fake_cid = "QmTestCid1234567890abcdef";

    Mock::given(method("POST"))
        .and(path("/api/v0/add"))
        .respond_with(mock_ipfs_add(fake_cid, "main.nr", "42"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(
        dir.path(),
        "test_circuit",
        "fn main(x: u64, y: pub u64) { assert(x != y); }",
    );

    cmd()
        .current_dir(dir.path())
        .args([
            "--ipfs-rpc-url",
            &mock_server.uri(),
            "new-compliance-definition",
            project.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(fake_cid))
        .stderr(predicate::str::contains("circuit compiled successfully"));

    // Default receipt should be written
    assert!(
        dir.path().join("receipt.json").exists(),
        "default receipt.json should be written"
    );
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
    let project = create_nargo_project(dir.path(), "test_circuit", "fn main() {}");

    cmd()
        .current_dir(dir.path())
        .args([
            "--ipfs-rpc-url",
            &mock_server.uri(),
            "new-compliance-definition",
            project.to_str().unwrap(),
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
        .respond_with(mock_ipfs_add(fake_cid, "main.nr", "10"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(dir.path(), "test_circuit", "fn main() {}");

    cmd()
        .current_dir(dir.path())
        .env("IPFS_RPC_URL", &mock_server.uri())
        .args(["new-compliance-definition", project.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(fake_cid));
}

// -- Receipt output --

#[tokio::test]
async fn output_flag_overrides_default_receipt_path() {
    let mock_server = MockServer::start().await;
    let fake_cid = "QmReceiptTestCid";

    Mock::given(method("POST"))
        .and(path("/api/v0/add"))
        .respond_with(mock_ipfs_add(fake_cid, "main.nr", "128"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(
        dir.path(),
        "test_circuit",
        "fn main(addr: pub Field) { assert(addr != 0); }",
    );
    let receipt_path = dir.path().join("custom-receipt.json");

    cmd()
        .current_dir(dir.path())
        .args([
            "--ipfs-rpc-url",
            &mock_server.uri(),
            "--output",
            receipt_path.to_str().unwrap(),
            "new-compliance-definition",
            project.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(fake_cid));

    // Custom path should exist, default should not
    assert!(receipt_path.exists());
    assert!(!dir.path().join("receipt.json").exists());

    let receipt: Value =
        serde_json::from_str(&std::fs::read_to_string(&receipt_path).unwrap()).unwrap();

    assert_eq!(receipt["command"], "new-compliance-definition");
    assert_eq!(receipt["data"]["cid"], fake_cid);
    assert_eq!(receipt["data"]["ipfs_size"], "128");
    assert!(receipt["timestamp"].as_str().unwrap().contains("T"));
    assert!(receipt["data"]["file_path"].as_str().unwrap().ends_with("main.nr"));
    assert!(receipt["data"]["project_dir"].as_str().is_some());
}

#[tokio::test]
async fn default_receipt_written_with_correct_contents() {
    let mock_server = MockServer::start().await;
    let fake_cid = "QmDefaultReceiptCid";

    Mock::given(method("POST"))
        .and(path("/api/v0/add"))
        .respond_with(mock_ipfs_add(fake_cid, "main.nr", "10"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(dir.path(), "test_circuit", "fn main() {}");

    cmd()
        .current_dir(dir.path())
        .args([
            "--ipfs-rpc-url",
            &mock_server.uri(),
            "new-compliance-definition",
            project.to_str().unwrap(),
        ])
        .assert()
        .success();

    let receipt_path = dir.path().join("receipt.json");
    assert!(receipt_path.exists(), "default receipt.json should be written");

    let receipt: Value =
        serde_json::from_str(&std::fs::read_to_string(&receipt_path).unwrap()).unwrap();

    assert_eq!(receipt["command"], "new-compliance-definition");
    assert_eq!(receipt["data"]["cid"], fake_cid);
    assert!(receipt["timestamp"].as_str().unwrap().contains("T"));
}
