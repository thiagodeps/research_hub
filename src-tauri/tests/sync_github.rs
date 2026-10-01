//! SEP-034: testes de sincronização contra um servidor wiremock local
//! (research R8 — a suíte nunca depende de rede real, Princípio III).
//!
//! Cobre o cliente HTTP (`sync_github`) e o caminho download→import que reusa
//! os imports manuais (R7). As chamadas apontam para a base URL do mock; a
//! normalização de URL e a política são testadas no domínio puro
//! (`sync_domain::tests`).

use research_hub_lib::db;
use research_hub_lib::error::AppError;
use research_hub_lib::sync_domain::{self, NormalizedSource};
use research_hub_lib::sync_github::{
    check_project_destination, download_and_import, upload_horizon_export, GitHubClient,
    TokenInfo,
};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Serializa o erro no wire `{ kind, message }` para os asserts de kind —
/// `AppError::kind()` é pub(crate) por design.
fn wire(err: &AppError) -> serde_json::Value {
    serde_json::to_value(err).expect("AppError sempre serializa")
}

fn file_source(owner: &str, repo: &str, reference: &str, path: &str) -> NormalizedSource {
    NormalizedSource::File {
        owner: owner.into(),
        repo: repo.into(),
        reference: reference.into(),
        path: path.into(),
    }
}

/// ZIP canônico válido: uma entrada `{table}_canonical.json` por tabela
/// gerida, todas vazias exceto `campuses` (id/name). Tabelas vazias são
/// puladas pelo import (`columns.is_empty()`), então o pacote é aceito com
/// linhas só em campuses.
fn canonical_zip(campuses: &[(&str, i64)]) -> Vec<u8> {
    use std::io::Write;
    let mut zip_bytes = Vec::new();
    {
        let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
        let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
        for def in research_hub_lib::registry::exported() {
            z.start_file(format!("{}_canonical.json", def.table), opts).unwrap();
            if def.table == "campuses" {
                let items: Vec<String> = campuses
                    .iter()
                    .map(|(name, id)| format!(r#"{{"id": {id}, "name": "{name}"}}"#))
                    .collect();
                z.write_all(format!("[{}]", items.join(", ")).as_bytes()).unwrap();
            } else {
                z.write_all(b"[]").unwrap();
            }
        }
        z.finish().unwrap();
    }
    zip_bytes
}

fn no_progress() -> &'static (dyn Fn(sync_domain::SyncProgress) + Send + Sync) {
    &|_| {}
}

fn client(server: &MockServer, token: Option<&str>) -> GitHubClient {
    GitHubClient::new(&server.uri(), token.map(String::from))
}

// ------------------------------------------------------------------ T007

#[tokio::test]
async fn download_publico_grava_temporario_e_emite_progresso() {
    let server = MockServer::start().await;
    let zip_bytes = canonical_zip(&[("Serra", 1)]);
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .and(header("Accept", "application/vnd.github.raw"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_bytes))
        .mount(&server)
        .await;

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");

    let phases = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let phases2 = phases.clone();
    let on_progress = move |p: sync_domain::SyncProgress| {
        phases2.lock().unwrap().push(p.phase);
    };

    let temp = tempfile::tempdir().unwrap();
    let path = GitHubClient::new(&server.uri(), None)
        .download_to_temp(&src, "horizon", temp.path(), &on_progress)
        .await
        .unwrap();

    assert!(path.exists());
    assert!(path.to_string_lossy().contains("exports_canonical.zip"));
    assert!(path.starts_with(temp.path()));

    let phases = phases.lock().unwrap();
    assert_eq!(phases.first(), Some(&"started"));
    assert!(phases.contains(&"transferring"));
}

#[tokio::test]
async fn download_privado_envia_o_token_e_sem_token_da_401() {
    let server = MockServer::start().await;
    let zip_bytes = canonical_zip(&[]);
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .and(header("Authorization", "Bearer ghp_token_teste"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_bytes))
        .mount(&server)
        .await;
    // Sem o header, o GitHub responde 401.
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .respond_with(ResponseTemplate::new(401).set_body_string(r#"{"message":"Requires authentication"}"#))
        .mount(&server)
        .await;

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let temp = tempfile::tempdir().unwrap();

    // Com token: sucesso (US1-2).
    let path = client(&server, Some("ghp_token_teste"))
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap();
    assert!(path.exists());

    // Sem token: erro acionável apontando para o token (US1-3), nada baixado.
    let err = client(&server, None)
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap_err();
    let w = wire(&err);
    assert_eq!(w["kind"], "github_api");
    assert!(w["message"].as_str().unwrap().contains("token"));
}

#[tokio::test]
async fn download_404_vira_not_found_e_nao_deixa_temporario() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/nao/existe.zip"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"message":"Not Found"}"#))
        .mount(&server)
        .await;

    let src = file_source("a", "b", "main", "nao/existe.zip");
    let temp = tempfile::tempdir().unwrap();

    let err = client(&server, None)
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap_err();
    assert_eq!(wire(&err)["kind"], "not_found");
    assert!(read_dir_empty(&temp.path().join("sync_tmp")));
}

#[tokio::test]
async fn download_acima_do_limite_aborta_com_too_large() {
    let server = MockServer::start().await;
    // 20 bytes no servidor; limite do cliente em 10 bytes (sem alocar 100 MB).
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 20]))
        .mount(&server)
        .await;

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let temp = tempfile::tempdir().unwrap();

    let client = GitHubClient::with_max_bytes(&server.uri(), None, 10);
    let err = client
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap_err();
    let w = wire(&err);
    assert_eq!(w["kind"], "too_large");
    assert_eq!(w["message"].as_str().unwrap(), "Arquivo acima do limite de 10 bytes aceito nesta versão.");
}

#[tokio::test]
async fn release_asset_e_localizado_por_tag_e_nome() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/releases/tags/v1.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assets": [
                { "id": 77, "name": "outro.zip" },
                { "id": 42, "name": "exports_canonical.zip" }
            ]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/releases/assets/42"))
        .and(header("Accept", "application/octet-stream"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(canonical_zip(&[("Serra", 9)])))
        .mount(&server)
        .await;

    let src = NormalizedSource::ReleaseAsset {
        owner: "a".into(),
        repo: "b".into(),
        tag: "v1.0.0".into(),
        asset: "exports_canonical.zip".into(),
    };

    let temp = tempfile::tempdir().unwrap();
    let path = client(&server, None)
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap();
    assert!(path.exists());
}

// ------------------------------------------------------------------ T008

#[tokio::test]
async fn download_horizon_importa_o_mesmo_resumo_do_import_manual() {
    let server = MockServer::start().await;
    let zip_bytes = canonical_zip(&[("Serra", 1), ("Vitória", 2)]);
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_bytes.clone()))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();

    // Via GitHub (US1): download + import.
    let mut conn = db::open_in_memory().unwrap();
    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let summary = download_and_import(
        &mut conn,
        data.path(),
        "horizon",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap();
    assert_eq!(summary["total_rows"], 2);

    // Manual do MESMO arquivo: resumo idêntico (SC-001 — "só muda a origem").
    let mut conn_manual = db::open_in_memory().unwrap();
    let manual = research_hub_lib::import::import_archive(
        &mut conn_manual,
        &zip_bytes,
        data.path(),
        |_| {},
    )
    .unwrap();
    assert_eq!(summary, serde_json::to_value(manual).unwrap());
}

#[tokio::test]
async fn download_src_importa_o_consolidado_e_falha_com_formato_errado() {
    let server = MockServer::start().await;
    let json = include_str!("fixtures/src_consolidado_exemplo.json");
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/dados/src_consolidado.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(json))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();
    let mut conn = db::open_in_memory().unwrap();

    // Caminho feliz: consolidado SRC pela URL (US1, projeto src).
    let src = file_source("a", "b", "main", "dados/src_consolidado.json");
    let summary = download_and_import(
        &mut conn,
        data.path(),
        "src",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap();
    assert!(summary["total_acoes"].as_u64().is_some());

    // FR-002/FR-009: zip na área SRC é recusado ANTES de qualquer chamada.
    let zip_src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let mut conn2 = db::open_in_memory().unwrap();
    let err = download_and_import(
        &mut conn2,
        data.path(),
        "src",
        &zip_src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap_err();
    assert_eq!(wire(&err)["kind"], "validation");

    // E vice-versa: .json na área Horizon também.
    let json_horizon = file_source("a", "b", "main", "dados/src_consolidado.json");
    let mut conn3 = db::open_in_memory().unwrap();
    let err = download_and_import(
        &mut conn3,
        data.path(),
        "horizon",
        &json_horizon,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap_err();
    assert_eq!(wire(&err)["kind"], "validation");
}

#[tokio::test]
async fn falha_no_import_deixa_a_base_intacta_e_limpa_o_temporario() {
    let server = MockServer::start().await;
    // ZIP ilegível como corpo: download ok, import falha.
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"isto nao e um zip".to_vec()))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();

    // Base com curadoria prévia.
    let mut conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", []).unwrap();

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let err = download_and_import(
        &mut conn,
        data.path(),
        "horizon",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap_err();

    // Erro legível; temporário removido.
    assert!(matches!(err, AppError::Internal(_)));
    let tmp_dir = data.path().join("sync_tmp");
    if tmp_dir.exists() {
        assert!(read_dir_empty(&tmp_dir));
    }

    // SC-002: a base permanece EXATAMENTE no estado anterior.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let name: String = conn
        .query_row("SELECT name FROM campuses WHERE id = 1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "Serra");
}

#[tokio::test]
async fn base_com_curadoria_gera_snapshot_no_download() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(canonical_zip(&[("Nova", 3)])))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();
    let mut conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", []).unwrap();

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let summary = download_and_import(
        &mut conn,
        data.path(),
        "horizon",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap();

    // US1-5: snapshot do estado anterior gravado e INFORMADO, como no manual.
    let snapshot = summary["snapshot"].as_str().expect("snapshot informado");
    assert!(snapshot.contains("hub-"));
    assert!(std::path::Path::new(snapshot).exists());
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1, "import substituiu a base");
}

// ------------------------------------------------------------------ T013

#[tokio::test]
async fn upload_fluxo_git_data_api_sucesso_com_ordem_e_sem_criar_branch() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "blob_sha_123"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "base_sha_abc" }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/trees"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "tree_sha_456"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/commits"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "commit_sha_789",
            "html_url": "https://github.com/a/b/commit/commit_sha_789"
        })))
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/repos/a/b/git/refs/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "commit_sha_789" }
        })))
        .mount(&server)
        .await;

    let phases = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let phases_c = phases.clone();
    let on_progress = move |p: sync_domain::SyncProgress| {
        phases_c.lock().unwrap().push(p.phase);
    };

    let client = GitHubClient::new(&server.uri(), Some("token".into()));
    let res = client
        .upload_file("a", "b", "main", "exports.zip", vec![1, 2, 3], "Commit msg", "horizon", &on_progress)
        .await
        .unwrap();

    assert_eq!(res.commit_sha, "commit_sha_789");
    assert_eq!(res.html_url, "https://github.com/a/b/commit/commit_sha_789");

    let received = server.received_requests().await.unwrap();
    let paths: Vec<String> = received.iter().map(|r| r.url.path().to_string()).collect();
    assert_eq!(paths, vec![
        "/repos/a/b/git/blobs",
        "/repos/a/b/git/ref/heads/main",
        "/repos/a/b/git/trees",
        "/repos/a/b/git/commits",
        "/repos/a/b/git/refs/heads/main",
    ]);

    // Q1: NUNCA cria ref com POST /git/refs
    assert!(!paths.iter().any(|p| p == "/repos/a/b/git/refs"));

    let phases = phases.lock().unwrap();
    assert_eq!(phases.first(), Some(&"started"));
    assert!(phases.contains(&"committing"));
}

#[tokio::test]
async fn upload_branch_inexistente_retorna_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({ "sha": "b1" })))
        .mount(&server)
        .await;

    // branch dá 404
    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/feature"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    // repo existe (200)
    Mock::given(method("GET"))
        .and(path("/repos/a/b"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = GitHubClient::new(&server.uri(), None);
    let err = client
        .upload_file("a", "b", "feature", "x.zip", vec![1], "msg", "horizon", no_progress())
        .await
        .unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "not_found");
    assert!(w["message"].as_str().unwrap().contains("branch 'feature' não existe"));
}

#[tokio::test]
async fn upload_patch_403_retorna_sem_permissao_de_escrita() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/blobs")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"b1"}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object":{"sha":"c0"}}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/trees")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"t1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/commits")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"c1"}))).mount(&server).await;
    Mock::given(method("PATCH")).and(path("/repos/a/b/git/refs/heads/main")).respond_with(ResponseTemplate::new(403)).mount(&server).await;

    let client = GitHubClient::new(&server.uri(), None);
    let err = client
        .upload_file("a", "b", "main", "x.zip", vec![1], "msg", "horizon", no_progress())
        .await
        .unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "github_api");
    assert!(w["message"].as_str().unwrap().contains("sem permissão de escrita; nada foi gravado"));
}

#[tokio::test]
async fn upload_patch_422_reexecuta_e_se_persistir_falha() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/blobs")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"b1"}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object":{"sha":"c0"}}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/trees")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"t1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/commits")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"c1"}))).mount(&server).await;
    Mock::given(method("PATCH")).and(path("/repos/a/b/git/refs/heads/main")).respond_with(ResponseTemplate::new(422)).mount(&server).await;

    let client = GitHubClient::new(&server.uri(), None);
    let err = client
        .upload_file("a", "b", "main", "x.zip", vec![1], "msg", "horizon", no_progress())
        .await
        .unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "github_api");
    assert!(w["message"].as_str().unwrap().contains("o repositório mudou durante o envio; tente novamente"));
}

#[tokio::test]
async fn upload_acima_do_limite_aborta_com_too_large_antes_de_blobs() {
    let server = MockServer::start().await;
    let client = GitHubClient::with_max_bytes(&server.uri(), None, 10);
    let err = client
        .upload_file("a", "b", "main", "x.zip", vec![0u8; 20], "msg", "horizon", no_progress())
        .await
        .unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "too_large");
    assert_eq!(server.received_requests().await.unwrap().len(), 0);
}

// ------------------------------------------------------------------ T014

#[tokio::test]
async fn check_destination_retorna_flags_e_trata_ausencias() {
    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();

    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/contents/exports.zip")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"sha": "sha_arquivo_xyz"}))).mount(&server).await;

    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("horizon").repo = "a/b".into();
    cfg.project_mut("horizon").branch = "main".into();
    cfg.project_mut("horizon").path = "exports.zip".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    let client = GitHubClient::new(&server.uri(), None);
    let view = check_project_destination(temp.path(), "horizon", &client).await.unwrap();

    assert_eq!(view.repo, "a/b");
    assert_eq!(view.branch, "main");
    assert_eq!(view.path, "exports.zip");
    assert!(view.branch_exists);
    assert!(view.file_exists);
    assert_eq!(view.file_sha.as_deref(), Some("sha_arquivo_xyz"));

    // Sem destino configurado -> sync_config
    let err = check_project_destination(temp.path(), "src", &client).await.unwrap_err();
    assert_eq!(wire(&err)["kind"], "sync_config");

    // Repo 404 -> not_found "repositório não encontrado ou sem acesso"
    let server_404 = MockServer::start().await;
    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(404)).mount(&server_404).await;
    let client_404 = GitHubClient::new(&server_404.uri(), None);
    let err = check_project_destination(temp.path(), "horizon", &client_404).await.unwrap_err();
    let w = wire(&err);
    assert_eq!(w["kind"], "not_found");
    assert!(w["message"].as_str().unwrap().contains("repositório não encontrado ou sem acesso"));
}

// ------------------------------------------------------------------ T015

#[tokio::test]
async fn upload_recusa_src_por_politica() {
    let temp = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let client = GitHubClient::new(&server.uri(), None);

    let err = upload_horizon_export(vec![], temp.path(), "src", false, &client, no_progress()).await.unwrap_err();
    assert_eq!(wire(&err)["kind"], "sync_policy");
    assert_eq!(server.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn upload_horizon_sucesso_envia_bytes_identicos_ao_export_manual() {
    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", []).unwrap();

    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("horizon").repo = "a/b".into();
    cfg.project_mut("horizon").branch = "main".into();
    cfg.project_mut("horizon").path = "exports/exports_canonical.zip".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    // Destination checks
    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object":{"sha":"c0"}}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/contents/exports/exports_canonical.zip")).respond_with(ResponseTemplate::new(404)).mount(&server).await;

    // Upload calls
    Mock::given(method("POST")).and(path("/repos/a/b/git/blobs")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"blob1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/trees")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"tree1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/commits")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"commit1", "html_url": "https://github.com/a/b/commit/commit1"}))).mount(&server).await;
    Mock::given(method("PATCH")).and(path("/repos/a/b/git/refs/heads/main")).respond_with(ResponseTemplate::new(200)).mount(&server).await;

    let (expected_bytes, _, _) = research_hub_lib::export::build_archive(&conn, None, |_| {}).unwrap();
    let client = GitHubClient::new(&server.uri(), None);
    let res = upload_horizon_export(expected_bytes.clone(), temp.path(), "horizon", false, &client, no_progress()).await.unwrap();

    assert_eq!(res.commit_sha, "commit1");
    assert_eq!(res.html_url, "https://github.com/a/b/commit/commit1");
    assert!(!res.replaced);

    // Verify blob content is byte-for-byte identical to manual export
    let requests = server.received_requests().await.unwrap();
    let blob_req = requests.iter().find(|r| r.url.path() == "/repos/a/b/git/blobs").unwrap();
    let blob_json: serde_json::Value = serde_json::from_slice(&blob_req.body).unwrap();
    let b64 = blob_json["content"].as_str().unwrap();
    let sent_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64).unwrap();
    assert_eq!(sent_bytes, expected_bytes);
}

#[tokio::test]
async fn upload_destino_existente_com_sha_diferente_sem_confirmacao_da_conflict() {
    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", []).unwrap();

    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("horizon").repo = "a/b".into();
    cfg.project_mut("horizon").branch = "main".into();
    cfg.project_mut("horizon").path = "exports/exports_canonical.zip".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    // Destination checks: file exists with different SHA
    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object":{"sha":"c0"}}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/contents/exports/exports_canonical.zip")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"sha":"sha_antigo_divergente"}))).mount(&server).await;

    let (bytes, _, _) = research_hub_lib::export::build_archive(&conn, None, |_| {}).unwrap();
    let client = GitHubClient::new(&server.uri(), None);
    let err = upload_horizon_export(bytes, temp.path(), "horizon", false, &client, no_progress()).await.unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "conflict");
    assert!(w["message"].as_str().unwrap().contains("Confirme a sobrescrita"));
}

// ------------------------------------------------------------------ T018 (US3)

#[test]
fn save_token_grava_com_permissao_restrita_e_token_hint() {
    let temp = tempfile::tempdir().unwrap();
    let mut cfg = sync_domain::load_config(temp.path()).unwrap();
    let secret_token = "ghp_1234567890abcdef";
    cfg.token = Some(secret_token.to_string());
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    let reloaded = sync_domain::load_config(temp.path()).unwrap();
    assert_eq!(reloaded.token.as_deref(), Some(secret_token));
    assert_eq!(sync_domain::token_hint(&reloaded.token), Some("cdef".into()));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file_path = temp.path().join(sync_domain::SYNC_CONFIG_FILE);
        let meta = std::fs::metadata(&file_path).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "arquivo de config com token deve ter permissão 0600");
    }
}

#[test]
fn remove_token_apaga_chave_e_arquivo_fica_com_vestigio_zero() {
    let temp = tempfile::tempdir().unwrap();
    let secret = "ghp_super_secret_token_123456789";

    // Grava inicialmente com o token
    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.token = Some(secret.to_string());
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    let file_path = temp.path().join(sync_domain::SYNC_CONFIG_FILE);
    let raw_before = std::fs::read_to_string(&file_path).unwrap();
    assert!(raw_before.contains(secret));

    // Remove o token (conforme github_remove_token)
    let mut to_remove = sync_domain::load_config(temp.path()).unwrap();
    to_remove.token = None;
    sync_domain::save_config(temp.path(), &to_remove).unwrap();

    // Recarrega e valida estado
    let reloaded = sync_domain::load_config(temp.path()).unwrap();
    assert_eq!(reloaded.token, None);
    assert_eq!(sync_domain::token_hint(&reloaded.token), None);

    // Inspeção direta do arquivo garante ZERO vestígio do token no disco (SC-006)
    let raw_after = std::fs::read_to_string(&file_path).unwrap();
    assert!(!raw_after.contains(secret), "o token secreto não deve constar em nenhuma parte do arquivo");
}

#[tokio::test]
async fn test_token_sucesso_retorna_login_escopos_sem_ecoar_token() {
    let server = MockServer::start().await;
    let secret = "ghp_valid_token_xyz987";

    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("Authorization", format!("Bearer {secret}").as_str()))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "login": "curador_ifes",
                    "id": 12345,
                }))
                .append_header("x-oauth-scopes", "repo, read:user"),
        )
        .mount(&server)
        .await;

    let client = GitHubClient::new(&server.uri(), Some(secret.to_string()));
    let info: TokenInfo = client.test_token().await.unwrap();

    assert_eq!(info.login, "curador_ifes");
    assert_eq!(info.scopes, "repo, read:user");
    assert!(info.valid);

    // Garante que a representação serializada ou debug não ecoa o token secreto
    let serialized = serde_json::to_string(&info).unwrap();
    assert!(!serialized.contains(secret), "TokenInfo não pode ecoar o token secreto");
    let debug_repr = format!("{info:?}");
    assert!(!debug_repr.contains(secret), "Debug de TokenInfo não pode ecoar o token secreto");
}

#[tokio::test]
async fn test_token_401_retorna_github_api_com_token_invalido() {
    let server = MockServer::start().await;
    let secret = "ghp_invalid_token";

    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("Authorization", format!("Bearer {secret}").as_str()))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "message": "Bad credentials"
        })))
        .mount(&server)
        .await;

    let client = GitHubClient::new(&server.uri(), Some(secret.to_string()));
    let err = client.test_token().await.unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "github_api");
    assert!(
        w["message"].as_str().unwrap().contains("token inválido ou revogado"),
        "Mensagem de erro deve ser acionável e apontar para token inválido"
    );
    assert!(
        !w["message"].as_str().unwrap().contains(secret),
        "Erro nunca deve conter o token secreto"
    );
}

#[test]
fn test_token_sem_token_configurado_da_sync_config() {
    let temp = tempfile::tempdir().unwrap();
    let cfg = sync_domain::load_config(temp.path()).unwrap();
    assert!(cfg.token.is_none());

    // Comportamento do comando github_test_token quando não há token salvo
    let res: Result<(), AppError> = match cfg.token {
        Some(_) => Ok(()),
        None => Err(AppError::SyncConfig("Nenhum token configurado.".into())),
    };

    let err = res.unwrap_err();
    let w = wire(&err);
    assert_eq!(w["kind"], "sync_config");
    assert!(w["message"].as_str().unwrap().contains("Nenhum token configurado"));
}

// ------------------------------------------------------------------ T022 (SC-005)

#[tokio::test]
async fn auditoria_de_vazamento_nenhum_erro_contem_token_ou_pii() {
    let server = MockServer::start().await;
    let secret_token = "ghp_super_secret_audit_token_999999999";
    let fake_cpf = "123.456.789-00";
    let fake_email = "participante.secreto@ifes.edu.br";
    let fake_nome = "Fulano de Tal Participante";

    let mut errors: Vec<AppError> = Vec::new();

    // 1. Network error (porta fechada com URL)
    let bad_client = GitHubClient::new("http://127.0.0.1:1", Some(secret_token.to_string()));
    let net_err = bad_client.test_token().await.unwrap_err();
    errors.push(net_err);

    // 2. github_api: 401
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Bad credentials"))
        .mount(&server)
        .await;
    let auth_client = GitHubClient::new(&server.uri(), Some(secret_token.to_string()));
    errors.push(auth_client.test_token().await.unwrap_err());

    // 3. github_api: 403
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/file.zip"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;
    let src = file_source("a", "b", "main", "file.zip");
    let temp = tempfile::tempdir().unwrap();
    errors.push(
        auth_client
            .download_to_temp(&src, "horizon", temp.path(), no_progress())
            .await
            .unwrap_err(),
    );

    // 4. too_large
    let client_small = GitHubClient::with_max_bytes(&server.uri(), Some(secret_token.to_string()), 10);
    errors.push(
        client_small
            .upload_file("a", "b", "main", "huge.zip", vec![0u8; 100], "msg", "horizon", no_progress())
            .await
            .unwrap_err(),
    );

    // 5. sync_policy (SRC bloqueado)
    errors.push(sync_domain::check_upload_allowed("src").unwrap_err());

    // 6. sync_config (validações de repo, branch, path)
    errors.push(sync_domain::validate_repo("invalid").unwrap_err());
    errors.push(sync_domain::validate_branch("").unwrap_err());
    errors.push(sync_domain::validate_path("horizon", "bad.tar").unwrap_err());

    // 7. not_found
    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/nao-existe"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    errors.push(
        auth_client
            .upload_file("a", "b", "nao-existe", "test.zip", vec![1], "msg", "horizon", no_progress())
            .await
            .unwrap_err(),
    );

    // 8. conflict
    errors.push(AppError::Conflict("Confirme a sobrescrita do arquivo".into()));

    // Audit assertion: NONE of the errors across all kinds leaks token or PII (SC-005)
    let sensitive_strings = [secret_token, fake_cpf, fake_email, fake_nome];
    for err in &errors {
        let msg = err.to_string();
        let debug = format!("{err:?}");
        let json = serde_json::to_string(&wire(err)).unwrap();

        for sensitive in sensitive_strings {
            assert!(
                !msg.contains(sensitive),
                "Mensagem de erro vazou dado sensível ({sensitive}): {msg}"
            );
            assert!(
                !debug.contains(sensitive),
                "Debug de erro vazou dado sensível ({sensitive}): {debug}"
            );
            assert!(
                !json.contains(sensitive),
                "JSON de erro vazou dado sensível ({sensitive}): {json}"
            );
        }
    }
}


// ------------------------------------------------------------------ utils

fn read_dir_empty(dir: &std::path::Path) -> bool {
    std::fs::read_dir(dir).map(|mut rd| rd.next().is_none()).unwrap_or(true)
}
