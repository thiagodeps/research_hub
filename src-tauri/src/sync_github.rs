//! Cliente GitHub da sincronização (SEP-034) — o único lugar que fala HTTP.
//!
//! Base URL injetável: produção usa `https://api.github.com`, os testes
//! apontam para um servidor wiremock local (research R8 — a suíte nunca
//! depende de rede real). O token viaja apenas no header `Authorization`;
//! nenhuma mensagem de erro o inclui (FR-006).
//!
//! Download: Contents API raw / Release Assets API (research R2), em chunks,
//! respeitando o limite de transferência (Q3). Upload: Git Data API
//! blob→tree→commit→ref (research R3) — a branch NUNCA é criada (Q1).

use crate::error::AppError;
use crate::sync_domain::{NormalizedSource, SyncProgress, MAX_TRANSFER_BYTES};
use base64::Engine;
use std::path::{Path, PathBuf};

/// Base URL padrão da API pública do GitHub.
pub const GITHUB_API: &str = "https://api.github.com";

/// Callback de progresso; o comando converte em evento Tauri `sync://progress`
/// (research R6). O módulo em si não conhece Tauri.
pub type OnProgress<'a> = &'a (dyn Fn(SyncProgress) + Send + Sync);

#[derive(Clone)]
pub struct GitHubClient {
    base_url: String,
    token: Option<String>,
    max_bytes: usize,
    http: reqwest::Client,
}

/// Resultado do envio (US2): confirmação com o commit gerado.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct UploadResult {
    pub commit_sha: String,
    pub html_url: String,
    pub replaced: bool,
}

/// Resultado da checagem prévia do destino (US2-2 / T014).
#[derive(serde::Serialize, Clone, Debug, Default)]
pub struct DestinationInfo {
    pub branch_exists: bool,
    pub file_exists: bool,
    pub file_sha: Option<String>,
}

/// Informações do token testado (US3 / T018).
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct TokenInfo {
    pub login: String,
    pub scopes: String,
    pub valid: bool,
}

impl GitHubClient {
    pub fn new(base_url: &str, token: Option<String>) -> Self {
        Self::with_max_bytes(base_url, token, MAX_TRANSFER_BYTES)
    }

    /// Limite ajustável para os testes (T007 usa um limite pequeno em vez de
    /// alocar 100 MB no servidor de mentira).
    pub fn with_max_bytes(base_url: &str, token: Option<String>, max_bytes: usize) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            max_bytes,
            http: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("cliente HTTP padrão deve ser construtível"),
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let req = req
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "research-hub");
        match &self.token {
            Some(t) => req.bearer_auth(t),
            None => req,
        }
    }

    /// Traduz uma resposta de erro do GitHub para `AppError` acionável
    /// (FR-007). O corpo do GitHub pode citar o path, mas nunca o token —
    /// que não é parte de nenhuma URL (R4).
    async fn api_error(&self, resp: reqwest::Response) -> AppError {
        let status = resp.status().as_u16();
        let hint = match status {
            401 => "o GitHub rejeitou o acesso: configure um token de acesso válido \
                    (ou use uma URL de repositório público)."
                .to_string(),
            403 => "acesso negado pelo GitHub: verifique as permissões do token \
                    (leitura para download; leitura e escrita para envio) ou o \
                    limite de requisições — tente mais tarde."
                .to_string(),
            _ => {
                let detail = resp
                    .json::<serde_json::Value>()
                    .await
                    .ok()
                    .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
                    .unwrap_or_default();
                if detail.is_empty() {
                    format!("o GitHub respondeu {status}; tente novamente")
                } else {
                    format!("o GitHub respondeu {status}: {detail}")
                }
            }
        };
        AppError::GithubApi { status, hint }
    }

    // ------------------------------------------------------------ download

    /// Baixa a origem normalizada para um arquivo temporário, em chunks
    /// (FR-008/SC-001). Em qualquer falha o temporário é removido antes de
    /// retornar — a base local nunca é tocada aqui.
    pub async fn download_to_temp(
        &self,
        source: &NormalizedSource,
        project: &str,
        temp_dir: &Path,
        on_progress: OnProgress<'_>,
    ) -> Result<PathBuf, AppError> {
        std::fs::create_dir_all(temp_dir)?;

        on_progress(SyncProgress {
            operation: "download",
            project: project.into(),
            phase: "started",
            bytes_done: 0,
            bytes_total: None,
        });

        // Localiza o asset de release (2 chamadas) ou monta a URL de contents.
        let (url, accept) = match source {
            NormalizedSource::File { path, reference, .. } => {
                let url = format!(
                    "{}/repos/{}/{}/contents/{}?ref={}",
                    self.base_url,
                    source.owner(),
                    source.repo(),
                    path,
                    reference
                );
                (url, "application/vnd.github.raw".to_string())
            }
            NormalizedSource::ReleaseAsset { tag, asset, .. } => {
                let releases_url = format!(
                    "{}/repos/{}/{}/releases/tags/{}",
                    self.base_url,
                    source.owner(),
                    source.repo(),
                    tag
                );
                let resp = self
                    .auth(self.http.get(&releases_url))
                    .send()
                    .await
                    .map_err(network_error)?;
                if !resp.status().is_success() {
                    return Err(self.api_error(resp).await);
                }
                let release: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
                let assets = release
                    .get("assets")
                    .and_then(|a| a.as_array())
                    .cloned()
                    .unwrap_or_default();
                let id = assets
                    .iter()
                    .find(|a| a.get("name").and_then(|n| n.as_str()) == Some(asset))
                    .and_then(|a| a.get("id").and_then(|i| i.as_u64()))
                    .ok_or_else(|| {
                        AppError::GithubApi {
                            status: 404,
                            hint: format!("o arquivo '{asset}' não existe na release '{tag}' deste repositório."),
                        }
                    })?;
                (
                    format!("{}/repos/{}/{}/releases/assets/{}", self.base_url, source.owner(), source.repo(), id),
                    "application/octet-stream".to_string(),
                )
            }
        };

        let resp = self
            .auth(self.http.get(&url))
            .header("Accept", accept)
            .send()
            .await
            .map_err(network_error)?;
        if !resp.status().is_success() {
            return Err(self.api_error(resp).await);
        }

        let total = resp.content_length();
        let file_name = format!(
            "sync-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
            source.file_name()
        );
        let dest = temp_dir.join(file_name);
        let mut file = std::fs::File::create(&dest)?;

        let mut resp = resp;
        let mut done: u64 = 0;
        loop {
            let chunk = match resp.chunk().await {
                Ok(Some(c)) => c,
                Ok(None) => break,
                Err(e) => {
                    let _ = std::fs::remove_file(&dest);
                    return Err(AppError::Network(format!(
                        "conexão interrompida durante o download: {e}"
                    )));
                }
            };
            done += chunk.len() as u64;
            if done as usize > self.max_bytes {
                let _ = std::fs::remove_file(&dest);
                return Err(AppError::TooLarge { limit_bytes: self.max_bytes });
            }
            use std::io::Write;
            file.write_all(&chunk)?;
            on_progress(SyncProgress {
                operation: "download",
                project: project.into(),
                phase: "transferring",
                bytes_done: done,
                bytes_total: total,
            });
        }

        Ok(dest)
    }

    // -------------------------------------------------------------- upload

    /// Lê a ponta atual da branch. 404 com repositório existente →
    /// `NotFound` (kind not_found) com contexto de branch — NUNCA cria a ref.
    async fn head_ref(&self, owner: &str, repo: &str, branch: &str) -> Result<String, AppError> {
        let url = format!("{}/repos/{}/{}/git/ref/heads/{}", self.base_url, owner, repo, branch);
        let resp = self.auth(self.http.get(&url)).send().await.map_err(network_error)?;
        match resp.status().as_u16() {
            200 => {
                let v: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
                Ok(v.pointer("/object/sha")
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string())
            }
            404 => {
                // Repositório existe? Distingue repo ausente de branch ausente.
                let repo_url = format!("{}/repos/{}/{}", self.base_url, owner, repo);
                let probe = self
                    .auth(self.http.get(&repo_url))
                    .send()
                    .await
                    .map_err(network_error)?;
                if probe.status().as_u16() == 404 {
                    Err(AppError::GithubApi {
                        status: 404,
                        hint: "repositório não encontrado ou sem acesso".into(),
                    })
                } else {
                    Err(AppError::GithubApi {
                        status: 404,
                        hint: format!(
                            "a branch '{branch}' não existe no repositório {owner}/{repo}. \
                             Crie a branch no GitHub antes de enviar — o aplicativo \
                             nunca cria branches automaticamente."
                        ),
                    })
                }
            }
            s => Err(AppError::GithubApi {
                status: s,
                hint: format!("o GitHub respondeu {s} ao ler a branch; tente novamente"),
            }),
        }
    }

    /// Fluxo Git Data API completo (research R3). `confirm_overwrite` chega
    /// validado pelo comando; aqui é só a mecânica.
    pub async fn upload_file(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        path: &str,
        bytes: Vec<u8>,
        message: &str,
        project: &str,
        on_progress: OnProgress<'_>,
    ) -> Result<UploadResult, AppError> {
        on_progress(SyncProgress {
            operation: "upload",
            project: project.into(),
            phase: "started",
            bytes_done: 0,
            bytes_total: Some(bytes.len() as u64),
        });

        // Limite ANTES de qualquer transferência (Q3/FR-010).
        if bytes.len() > self.max_bytes {
            return Err(AppError::TooLarge { limit_bytes: self.max_bytes });
        }

        on_progress(SyncProgress {
            operation: "upload",
            project: project.into(),
            phase: "committing",
            bytes_done: bytes.len() as u64,
            bytes_total: Some(bytes.len() as u64),
        });

        // 1. blob
        let blob_url = format!("{}/repos/{}/{}/git/blobs", self.base_url, owner, repo);
        let content = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let resp = self
            .auth(self.http.post(&blob_url))
            .json(&serde_json::json!({ "content": content, "encoding": "base64" }))
            .send()
            .await
            .map_err(network_error)?;
        if !resp.status().is_success() {
            return Err(self.api_error(resp).await);
        }
        let blob: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
        let blob_sha = blob
            .get("sha")
            .and_then(|s| s.as_str())
            .ok_or_else(|| AppError::Network("resposta do blob sem sha".into()))?
            .to_string();

        // 2..5 com uma retomada: a ponta pode mudar entre a leitura e o PATCH
        // (422). Segunda falha → erro acionável, nada gravado pelo app.
        let mut attempt = 0;
        loop {
            let base_sha = self.head_ref(owner, repo, branch).await?;

            // tree
            let tree_url = format!("{}/repos/{}/{}/git/trees", self.base_url, owner, repo);
            let resp = self
                .auth(self.http.post(&tree_url))
                .json(&serde_json::json!({
                    "base_tree": base_sha,
                    "tree": [{ "path": path, "mode": "100644", "type": "blob", "sha": blob_sha }],
                }))
                .send()
                .await
                .map_err(network_error)?;
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }
            let tree: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
            let tree_sha = tree
                .get("sha")
                .and_then(|s| s.as_str())
                .ok_or_else(|| AppError::Network("resposta da tree sem sha".into()))?
                .to_string();

            // commit
            let commit_url = format!("{}/repos/{}/{}/git/commits", self.base_url, owner, repo);
            let resp = self
                .auth(self.http.post(&commit_url))
                .json(&serde_json::json!({ "message": message, "tree": tree_sha, "parents": [base_sha] }))
                .send()
                .await
                .map_err(network_error)?;
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }
            let commit: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
            let commit_sha = commit
                .get("sha")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string();
            let html_url = commit
                .get("html_url")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string();

            // avanço da ref
            let ref_url = format!("{}/repos/{}/{}/git/refs/heads/{}", self.base_url, owner, repo, branch);
            let resp = self
                .auth(self.http.patch(&ref_url))
                .json(&serde_json::json!({ "sha": commit_sha, "force": false }))
                .send()
                .await
                .map_err(network_error)?;
            let status = resp.status().as_u16();
            if status == 422 && attempt == 0 {
                attempt += 1;
                continue; // ponta mudou; re-ler e refazer tree/commit/ref
            }
            if status == 422 {
                return Err(AppError::GithubApi {
                    status: 422,
                    hint: "o repositório mudou durante o envio; tente novamente".into(),
                });
            }
            if status == 403 {
                return Err(AppError::GithubApi {
                    status: 403,
                    hint: "sem permissão de escrita; nada foi gravado".into(),
                });
            }
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }

            return Ok(UploadResult {
                commit_sha,
                html_url,
                replaced: false,
            });
        }
    }

    /// Checagem prévia do destino (T014): flags em vez de erro para branch/
    /// arquivo ausentes — quem falha é o upload. Repo ausente → NotFound.
    pub async fn check_destination(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        path: &str,
    ) -> Result<DestinationInfo, AppError> {
        let mut info = DestinationInfo::default();

        let repo_url = format!("{}/repos/{}/{}", self.base_url, owner, repo);
        let repo_resp = self
            .auth(self.http.get(&repo_url))
            .send()
            .await
            .map_err(network_error)?;
        if repo_resp.status().as_u16() == 404 {
            return Err(AppError::GithubApi {
                status: 404,
                hint: "repositório não encontrado ou sem acesso".into(),
            });
        }
        if !repo_resp.status().is_success() {
            return Err(self.api_error(repo_resp).await);
        }

        let ref_url = format!("{}/repos/{}/{}/git/ref/heads/{}", self.base_url, owner, repo, branch);
        let ref_resp = self.auth(self.http.get(&ref_url)).send().await.map_err(network_error)?;
        info.branch_exists = ref_resp.status().is_success();
        if !info.branch_exists && ref_resp.status().as_u16() != 404 {
            return Err(self.api_error(ref_resp).await);
        }

        if info.branch_exists {
            let contents_url = format!(
                "{}/repos/{}/{}/contents/{}?ref={}",
                self.base_url, owner, repo, path, branch
            );
            let resp = self
                .auth(self.http.get(&contents_url))
                .send()
                .await
                .map_err(network_error)?;
            match resp.status().as_u16() {
                200 => {
                    let v: serde_json::Value = resp
                        .json()
                        .await
                        .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
                    info.file_exists = true;
                    info.file_sha = v.get("sha").and_then(|s| s.as_str()).map(String::from);
                }
                404 => {}
                s => {
                    let _ = &contents_url;
                    return Err(AppError::GithubApi {
                        status: s,
                        hint: format!("o GitHub respondeu {s} ao ler o arquivo de destino; tente novamente"),
                    });
                }
            }
        }

        Ok(info)
    }

    // ---------------------------------------------------------------- token

    /// Valida o token contra `GET /user` (US3). O valor nunca é ecoado.
    pub async fn test_token(&self) -> Result<TokenInfo, AppError> {
        let url = format!("{}/user", self.base_url);
        let resp = self.auth(self.http.get(&url)).send().await.map_err(network_error)?;
        if resp.status().as_u16() == 401 {
            return Err(AppError::GithubApi {
                status: 401,
                hint: "token inválido ou revogado; salve um token atualizado".into(),
            });
        }
        if !resp.status().is_success() {
            return Err(self.api_error(resp).await);
        }
        let scopes = resp
            .headers()
            .get("x-oauth-scopes")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
        Ok(TokenInfo {
            login: v.get("login").and_then(|l| l.as_str()).unwrap_or_default().to_string(),
            scopes,
            valid: true,
        })
    }
}

/// reqwest::Error → `network` com dica acionável (FR-007). O Display do erro
/// não contém o token (ele só existe no header, que o Display não imprime).
fn network_error(e: reqwest::Error) -> AppError {
    AppError::Network(format!(
        "sem acesso a github.com — verifique sua conexão com a internet e tente de novo ({e})"
    ))
}

// --------------------------------------------------------------- importação

/// Download + import em um passo (T008): origem validada por projeto, bytes em
/// temporário, import EXISTENTE reutilizado (R7 — mesmas validações, snapshot
/// e tudo-ou-nada), temporário removido em qualquer desfecho.
///
/// Reusar o import manual por caminho de arquivo garante SC-004 por construção.
pub async fn download_and_import(
    conn: &mut rusqlite::Connection,
    data_dir: &Path,
    project: &str,
    source: &NormalizedSource,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<serde_json::Value, AppError> {
    crate::sync_domain::ensure_known_project(project)?;

    // FR-002/FR-009: cada projeto aceita só o seu formato.
    let expected = if project == "horizon" { "zip" } else { "json" };
    let name = source.file_name().to_ascii_lowercase();
    if !name.ends_with(&format!(".{expected}")) {
        return Err(AppError::Validation(format!(
            "O projeto {project} espera um export .{expected}; a URL aponta para '{}'.",
            source.file_name()
        )));
    }

    let temp_dir = data_dir.join("sync_tmp");
    let temp_path = client.download_to_temp(source, project, &temp_dir, on_progress).await?;

    on_progress(SyncProgress {
        operation: "download",
        project: project.into(),
        phase: "importing",
        bytes_done: 0,
        bytes_total: None,
    });

    let result: Result<serde_json::Value, AppError> = match project {
        "horizon" => {
            let bytes = std::fs::read(&temp_path)?;
            let mut conn_ref = conn;
            crate::import::import_archive(&mut conn_ref, &bytes, data_dir, |_t| {})
                .and_then(|summary| {
                    serde_json::to_value(summary)
                        .map_err(|e| AppError::Internal(format!("resumo ilegível: {e}")))
                })
        }
        "src" => {
            let text = std::fs::read_to_string(&temp_path)?;
            let mut conn_ref = conn;
            crate::src_domain::import(&mut conn_ref, &text, data_dir)
                .and_then(|summary| {
                    serde_json::to_value(summary)
                        .map_err(|e| AppError::Internal(format!("resumo ilegível: {e}")))
                })
        }
        _ => unreachable!("projeto validado acima"),
    };

    let _ = std::fs::remove_file(&temp_path);
    result
}

/// Checagem do destino configurado para um projeto (T014).
pub async fn check_project_destination(
    data_dir: &Path,
    project: &str,
    client: &GitHubClient,
) -> Result<crate::sync_domain::CheckDestinationView, AppError> {
    crate::sync_domain::ensure_known_project(project)?;
    let file = crate::sync_domain::load_config(data_dir)?;
    let empty = crate::sync_domain::ProjectSyncConfig::default();
    let cfg = file.project(project).unwrap_or(&empty);
    if cfg.repo.is_empty() || cfg.branch.is_empty() || cfg.path.is_empty() {
        return Err(AppError::SyncConfig(
            "Destino não configurado para este projeto.".into(),
        ));
    }
    let (owner, repo) = cfg
        .repo
        .split_once('/')
        .ok_or_else(|| AppError::SyncConfig("Repositório deve estar no formato 'owner/name'.".into()))?;

    let info = client.check_destination(owner, repo, &cfg.branch, &cfg.path).await?;
    Ok(crate::sync_domain::CheckDestinationView {
        repo: cfg.repo.clone(),
        branch: cfg.branch.clone(),
        path: cfg.path.clone(),
        branch_exists: info.branch_exists,
        file_exists: info.file_exists,
        file_sha: info.file_sha,
    })
}

/// Envio do export canônico para o GitHub (T015).
pub async fn upload_horizon_export(
    bytes: Vec<u8>,
    data_dir: &Path,
    project: &str,
    confirm_overwrite: bool,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<UploadResult, AppError> {
    crate::sync_domain::check_upload_allowed(project)?;

    let file = crate::sync_domain::load_config(data_dir)?;
    let empty = crate::sync_domain::ProjectSyncConfig::default();
    let cfg = file.project(project).unwrap_or(&empty);
    if cfg.repo.is_empty() || cfg.branch.is_empty() || cfg.path.is_empty() {
        return Err(AppError::SyncConfig(
            "Destino não configurado para este projeto.".into(),
        ));
    }
    let (owner, repo) = cfg
        .repo
        .split_once('/')
        .ok_or_else(|| AppError::SyncConfig("Repositório deve estar no formato 'owner/name'.".into()))?;

    // Limite antes de qualquer chamada HTTP (Q3)
    if bytes.len() > client.max_bytes {
        return Err(AppError::TooLarge { limit_bytes: client.max_bytes });
    }

    // Checa sobrescrita se o arquivo já existir com sha diferente (US2-2)
    let dest_info = client.check_destination(owner, repo, &cfg.branch, &cfg.path).await?;
    if dest_info.file_exists {
        let current_sha = dest_info.file_sha.as_deref().unwrap_or_default();
        let new_sha = crate::sync_domain::git_blob_sha(&bytes);
        if current_sha != new_sha && !confirm_overwrite {
            return Err(AppError::Conflict(
                "O arquivo de destino já existe no repositório com conteúdo diferente. Confirme a sobrescrita para prosseguir.".into(),
            ));
        }
    }

    let message = "Export canônico ResearchHub";
    let mut res = client
        .upload_file(owner, repo, &cfg.branch, &cfg.path, bytes, message, project, on_progress)
        .await?;
    res.replaced = dest_info.file_exists;
    Ok(res)
}

