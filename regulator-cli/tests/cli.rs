use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
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

/// Dummy chain args for publish tests that fail before reaching chain operations.
const PUBLISH_CHAIN_ARGS: [&str; 6] = [
    "--rpc-url",
    "http://localhost:8545",
    "--private-key",
    "0xdeadbeef",
    "--compliance-definition",
    "0x0000000000000000000000000000000000000001",
];

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

// -- New compliance definition command --

#[test]
fn new_compliance_definition_requires_rpc_url() {
    cmd()
        .args([
            "new-compliance-definition",
            "--private-key",
            "0xdeadbeef",
            "--regulator",
            "0x1234",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--rpc-url"));
}

#[test]
fn new_compliance_definition_requires_private_key() {
    cmd()
        .args([
            "new-compliance-definition",
            "--rpc-url",
            "http://localhost:8545",
            "--regulator",
            "0x1234",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--private-key"));
}

#[test]
fn new_compliance_definition_requires_regulator() {
    cmd()
        .args([
            "new-compliance-definition",
            "--rpc-url",
            "http://localhost:8545",
            "--private-key",
            "0xdeadbeef",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--regulator"));
}

#[test]
fn new_compliance_definition_rejects_invalid_contract_dir() {
    cmd()
        .args([
            "new-compliance-definition",
            "--rpc-url",
            "http://localhost:8545",
            "--private-key",
            "0xdeadbeef",
            "--regulator",
            "0x1234",
            "--contract-dir",
            "/tmp/nonexistent-foundry-project",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("forge create failed"));
}

// -- Publish command --

#[test]
fn publish_requires_path_argument() {
    cmd()
        .args(["publish", "--rpc-url", "http://localhost:8545", "--private-key", "0xdead", "--compliance-definition", "0x1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("DIR"));
}

#[test]
fn publish_requires_rpc_url() {
    cmd()
        .args(["publish", "--private-key", "0xdead", "--compliance-definition", "0x1", "/tmp/x"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--rpc-url"));
}

#[test]
fn publish_requires_private_key() {
    cmd()
        .args(["publish", "--rpc-url", "http://localhost:8545", "--compliance-definition", "0x1", "/tmp/x"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--private-key"));
}

#[test]
fn publish_requires_compliance_definition() {
    cmd()
        .args(["publish", "--rpc-url", "http://localhost:8545", "--private-key", "0xdead", "/tmp/x"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--compliance-definition"));
}

#[test]
fn publish_rejects_nonexistent_directory() {
    let mut args = vec!["publish"];
    args.extend_from_slice(&PUBLISH_CHAIN_ARGS);
    args.push("/tmp/nonexistent-noir-project");

    cmd()
        .args(&args)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn publish_rejects_directory_without_nargo_toml() {
    let dir = tempfile::tempdir().unwrap();
    let mut args = vec!["publish"];
    args.extend_from_slice(&PUBLISH_CHAIN_ARGS);
    args.push(dir.path().to_str().unwrap());

    cmd()
        .args(&args)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no Nargo.toml found"));
}

#[test]
fn publish_rejects_invalid_circuit() {
    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(dir.path(), "bad_circuit", "this is not valid noir");
    let mut args = vec!["publish"];
    args.extend_from_slice(&PUBLISH_CHAIN_ARGS);
    args.push(project.to_str().unwrap());

    cmd()
        .args(&args)
        .assert()
        .failure()
        .stderr(predicate::str::contains("circuit validation failed"));
}

#[tokio::test]
async fn publish_reports_ipfs_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v0/add"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let project = create_nargo_project(
        dir.path(),
        "test_circuit",
        "fn main(x: u64, y: pub u64) { assert(x != y); }",
    );

    let ipfs_uri = mock_server.uri();
    let project_str = project.to_str().unwrap();
    let mut args = vec![
        "--ipfs-rpc-url",
        &ipfs_uri,
        "publish",
    ];
    args.extend_from_slice(&PUBLISH_CHAIN_ARGS);
    args.push(project_str);

    cmd()
        .current_dir(dir.path())
        .args(&args)
        .assert()
        .failure()
        .stderr(predicate::str::contains("IPFS"));
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
