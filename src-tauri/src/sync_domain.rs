//! Lógica pura da sincronização com o GitHub (SEP-034).
//!
//! Nada aqui conhece Tauri nem HTTP: parsing de URL, política de privacidade
//! (FR-005), limites de tamanho (Q3) e o armazenamento local da configuração
//! (`sync_config.json`, R4/R10). Testável sem rede e sem janela — o cliente
//! HTTP de verdade mora em `sync_github.rs`.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Limite desta entrega para download e envio (Q1/Q3 da spec: 100 MB).
pub const MAX_TRANSFER_BYTES: usize = 100 * 1024 * 1024;

/// Evento de progresso `sync://progress` (research R6, FR-008). Fases:
/// download started→transferring→importing→done|failed;
/// upload started→committing→done|failed.
#[derive(Clone, Debug, serde::Serialize)]
pub struct SyncProgress {
    pub operation: &'static str,
    pub project: String,
    pub phase: &'static str,
    pub bytes_done: u64,
    pub bytes_total: Option<u64>,
}

/// Arquivo de configuração no app-data (research R4).
pub const SYNC_CONFIG_FILE: &str = "sync_config.json";

/// Projetos conhecidos. O envio do SRC é bloqueado por política (FR-005),
/// mas a estrutura existe para os dois — a decisão é reversível sem
/// redesenho (Q1 da discussão do curador).
pub const PROJECTS: [&str; 2] = ["horizon", "src"];

/// Destino/envio de UM projeto (FR-004: configurações separadas — os
/// repositórios dos dois projetos são gits separados).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ProjectSyncConfig {
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_source_url: Option<String>,
}

/// Conteúdo completo de `sync_config.json`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SyncConfigFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub projects: HashMap<String, ProjectSyncConfig>,
}

/// Default manual: `version` começa em 1 (o derive daria 0).
impl Default for SyncConfigFile {
    fn default() -> Self {
        Self {
            version: default_version(),
            token: None,
            projects: HashMap::new(),
        }
    }
}

fn default_version() -> u32 {
    1
}

impl SyncConfigFile {
    pub fn project(&self, project: &str) -> Option<&ProjectSyncConfig> {
        self.projects.get(project)
    }

    pub fn project_mut(&mut self, project: &str) -> &mut ProjectSyncConfig {
        self.projects.entry(project.to_string()).or_default()
    }
}

/// Últimos 4 caracteres do token para a UI (FR-006: o valor completo nunca
/// atravessa IPC de volta). Sem token → `None`.
pub fn token_hint(token: &Option<String>) -> Option<String> {
    token.as_ref().map(|t| {
        let chars: Vec<char> = t.chars().collect();
        let tail: String = chars
            .iter()
            .rev()
            .take(4)
            .rev()
            .collect();
        tail
    })
}

// --------------------------------------------------------------- validação

/// `owner/name`, sem espaços, ambos não vazios (data-model.md).
pub fn validate_repo(repo: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|p| p.trim().is_empty())
        || repo.split_whitespace().count() != 1
    {
        return Err(AppError::SyncConfig(
            "Repositório deve estar no formato 'owner/name' (ex.: 'rafaeldeps/research-sync').".into(),
        ));
    }
    Ok(())
}

/// Branch não vazia, sem espaços, sem começar com `-` (data-model.md).
pub fn validate_branch(branch: &str) -> Result<(), AppError> {
    if branch.trim().is_empty()
        || branch.split_whitespace().count() != 1
        || branch.starts_with('-')
    {
        return Err(AppError::SyncConfig(
            "Branch não pode ficar vazia, conter espaços ou começar com '-'.".into(),
        ));
    }
    Ok(())
}

/// Caminho não vazio, sem `..`, e `.zip` para o Horizon (data-model.md).
pub fn validate_path(project: &str, path: &str) -> Result<(), AppError> {
    if path.trim().is_empty() {
        return Err(AppError::SyncConfig(
            "Caminho do arquivo no repositório não pode ficar vazio.".into(),
        ));
    }
    if path.split('/').any(|seg| seg == "..") {
        return Err(AppError::SyncConfig(
            "Caminho não pode conter '..'.".into(),
        ));
    }
    if project == "horizon" && !path.to_ascii_lowercase().ends_with(".zip") {
        return Err(AppError::SyncConfig(
            "O export do Horizon é o zip canônico: o caminho deve terminar em '.zip'.".into(),
        ));
    }
    Ok(())
}

/// Validação completa do destino de um projeto.
pub fn validate_project_config(
    project: &str,
    config: &ProjectSyncConfig,
) -> Result<(), AppError> {
    ensure_known_project(project)?;
    validate_repo(&config.repo)?;
    validate_branch(&config.branch)?;
    validate_path(project, &config.path)
}

pub fn ensure_known_project(project: &str) -> Result<(), AppError> {
    if PROJECTS.contains(&project) {
        Ok(())
    } else {
        Err(AppError::SyncConfig(format!(
            "Projeto desconhecido: {project}"
        )))
    }
}

/// Política de dados pessoais para o envio (FR-005, decisão Option B): o
/// consolidado do SRC contém PII direta e NÃO sobe nesta entrega — erro SEM
/// qualquer chamada de rede. Bloqueio é política por projeto, reversível no
/// futuro sem redesenho (regra 1 do data-model.md).
pub fn check_upload_allowed(project: &str) -> Result<(), AppError> {
    match project {
        "horizon" => Ok(()),
        "src" => Err(AppError::SyncPolicy(
            "O consolidado do SRC contém dados pessoais (CPF, e-mail) e não \
             pode ser enviado ao GitHub nesta versão. O export local continua \
             disponível normalmente."
                .into(),
        )),
        other => Err(AppError::SyncConfig(format!(
            "Projeto desconhecido: {other}"
        ))),
    }
}

// ------------------------------------------------------------------- URLs

/// Origem de download normalizada (research R2): um arquivo em branch/ref ou
/// um asset de release.
#[derive(Clone, Debug, PartialEq)]
pub enum NormalizedSource {
    /// Contents API: `GET /repos/{owner}/{repo}/contents/{path}?ref={reference}`
    File {
        owner: String,
        repo: String,
        reference: String,
        path: String,
    },
    /// Release Assets API: localizar o asset por tag+nome e baixar em octet-stream.
    ReleaseAsset {
        owner: String,
        repo: String,
        tag: String,
        asset: String,
    },
}

impl NormalizedSource {
    /// Extensão do arquivo de origem (".zip"/".json") para validar contra o projeto.
    pub fn file_name(&self) -> &str {
        match self {
            Self::File { path, .. } => path.rsplit('/').next().unwrap_or(path),
            Self::ReleaseAsset { asset, .. } => asset,
        }
    }

    pub fn owner(&self) -> &str {
        match self {
            Self::File { owner, .. } | Self::ReleaseAsset { owner, .. } => owner,
        }
    }

    pub fn repo(&self) -> &str {
        match self {
            Self::File { repo, .. } | Self::ReleaseAsset { repo, .. } => repo,
        }
    }
}

/// Aceita as três formas declaradas na spec (raw, blob, release asset) e
/// normaliza para a Contents/Assets API. Qualquer outra URL → `validation`.
pub fn normalize_url(url: &str) -> Result<NormalizedSource, AppError> {
    let parsed = url::Url::parse(url)
        .map_err(|_| AppError::Validation(format!("URL inválida: {url}")))?;

    if parsed.scheme() != "https" {
        return Err(AppError::Validation(
            "Use uma URL https de arquivo do GitHub (raw, blob ou release asset).".into(),
        ));
    }

    let segments: Vec<String> = parsed
        .path_segments()
        .map(|s| s.map(|p| p.to_string()).collect())
        .unwrap_or_default();

    match parsed.host_str() {
        Some("raw.githubusercontent.com") => {
            // /{owner}/{repo}/{ref}/{path...}
            if segments.len() < 4 {
                return Err(AppError::Validation(
                    "URL raw esperada: https://raw.githubusercontent.com/{owner}/{repo}/{ref}/{caminho}".into(),
                ));
            }
            Ok(NormalizedSource::File {
                owner: segments[0].clone(),
                repo: segments[1].clone(),
                reference: segments[2].clone(),
                path: segments[3..].join("/"),
            })
        }
        Some("github.com") => {
            match segments.as_slice() {
                [owner, repo, kind, reference, path @ ..] if kind == "blob" && !path.is_empty() => {
                    Ok(NormalizedSource::File {
                        owner: owner.clone(),
                        repo: repo.clone(),
                        reference: reference.clone(),
                        path: path.join("/"),
                    })
                }
                [owner, repo, k1, k2, tag, asset] if k1 == "releases" && k2 == "download" => {
                    Ok(NormalizedSource::ReleaseAsset {
                        owner: owner.clone(),
                        repo: repo.clone(),
                        tag: tag.clone(),
                        asset: asset.clone(),
                    })
                }
                _ => Err(AppError::Validation(
                    "URL do GitHub esperada: .../blob/{ref}/{caminho} ou .../releases/download/{tag}/{arquivo}".into(),
                )),
            }
        }
        _ => Err(AppError::Validation(
            "Só URLs de github.com e raw.githubusercontent.com são aceitas nesta versão.".into(),
        )),
    }
}

// ------------------------------------------------------------- persistência

/// Visão da configuração que atravessa o IPC (T005). O valor completo do
/// token NUNCA retorna ao front-end — só presença + últimos 4 caracteres
/// (FR-006, contracts/ipc-commands.md).
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SyncConfigView {
    pub repo: String,
    pub branch: String,
    pub path: String,
    pub last_source_url: Option<String>,
    pub has_token: bool,
    pub token_hint: Option<String>,
}

/// Visão do destino para o front-end (contracts/ipc-commands.md).
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct CheckDestinationView {
    pub repo: String,
    pub branch: String,
    pub path: String,
    pub branch_exists: bool,
    pub file_exists: bool,
    pub file_sha: Option<String>,
}

pub fn sha1_hex(data: &[u8]) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }
    format!("{h0:08x}{h1:08x}{h2:08x}{h3:08x}{h4:08x}")
}

/// Calcula o Git blob SHA-1 canônico (`blob <len>\0<bytes>`).
pub fn git_blob_sha(bytes: &[u8]) -> String {
    let header = format!("blob {}\0", bytes.len());
    let mut payload = Vec::with_capacity(header.len() + bytes.len());
    payload.extend_from_slice(header.as_bytes());
    payload.extend_from_slice(bytes);
    sha1_hex(&payload)
}


/// Monta a visão de um projeto a partir do arquivo carregado. Projeto sem
/// configuração salva → campos vazios (não é erro: é o primeiro uso).
pub fn config_view(project: &str, file: &SyncConfigFile) -> Result<SyncConfigView, AppError> {
    ensure_known_project(project)?;
    let empty = ProjectSyncConfig::default();
    let cfg = file.project(project).unwrap_or(&empty);
    Ok(SyncConfigView {
        repo: cfg.repo.clone(),
        branch: cfg.branch.clone(),
        path: cfg.path.clone(),
        last_source_url: cfg.last_source_url.clone(),
        has_token: file.token.is_some(),
        token_hint: token_hint(&file.token),
    })
}

/// Salva o destino de um projeto após validar (regras do data-model.md).
/// O token e os demais projetos são preservados no arquivo.
pub fn set_project_config(
    data_dir: &Path,
    project: &str,
    repo: &str,
    branch: &str,
    path: &str,
) -> Result<SyncConfigFile, AppError> {
    let mut file = load_config(data_dir)?;
    let cfg = ProjectSyncConfig {
        repo: repo.trim().to_string(),
        branch: branch.trim().to_string(),
        path: path.trim().to_string(),
        last_source_url: file.project(project).and_then(|p| p.last_source_url.clone()),
    };
    validate_project_config(project, &cfg)?;
    file.projects.insert(project.to_string(), cfg);
    save_config(data_dir, &file)?;
    Ok(file)
}

pub fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join(SYNC_CONFIG_FILE)
}

/// Lê `sync_config.json`. Arquivo ausente → configuração vazia default;
/// conteúdo ilegível → `sync_config` (não corrompemos por cima sem avisar).
pub fn load_config(data_dir: &Path) -> Result<SyncConfigFile, AppError> {
    let path = config_path(data_dir);
    if !path.exists() {
        return Ok(SyncConfigFile::default());
    }
    let text = std::fs::read_to_string(&path)?;
    serde_json::from_str(&text)
        .map_err(|e| AppError::SyncConfig(format!("configuração de sincronização ilegível: {e}")))
}

/// Gravação atômica (R10): escreve `.tmp` + rename, e restringe a permissão
/// em Unix (0600) — o arquivo contém o token.
pub fn save_config(data_dir: &Path, config: &SyncConfigFile) -> Result<(), AppError> {
    std::fs::create_dir_all(data_dir)?;
    let path = config_path(data_dir);
    let tmp = data_dir.join(format!("{SYNC_CONFIG_FILE}.tmp"));
    let json = serde_json::to_string_pretty(config)?;

    std::fs::write(&tmp, json.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))?;
    }

    std::fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    // ------------------------------------------------------------- T004

    #[test]
    fn config_ausente_vira_default_sem_erro() {
        let d = dir();
        let cfg = load_config(d.path()).unwrap();
        assert_eq!(cfg.version, 1);
        assert!(cfg.token.is_none());
        assert!(cfg.projects.is_empty());
    }

    #[test]
    fn config_salva_e_recarrega_com_token_e_projetos() {
        let d = dir();
        let mut cfg = SyncConfigFile::default();
        cfg.token = Some("ghp_abc".into());
        cfg.project_mut("horizon").repo = "rafaeldeps/research-sync".into();
        cfg.project_mut("horizon").branch = "main".into();
        cfg.project_mut("horizon").path = "exports/exports_canonical.zip".into();
        cfg.project_mut("src").repo = "rafaeldeps/src-sync".into();
        save_config(d.path(), &cfg).unwrap();

        let loaded = load_config(d.path()).unwrap();
        assert_eq!(loaded, cfg);
        assert_eq!(loaded.project("src").unwrap().repo, "rafaeldeps/src-sync");
    }

    #[test]
    fn config_gravacao_e_atomica_e_deixa_sem_tmp() {
        let d = dir();
        let cfg = SyncConfigFile::default();
        save_config(d.path(), &cfg).unwrap();
        assert!(config_path(d.path()).exists());
        assert!(!d.path().join(format!("{SYNC_CONFIG_FILE}.tmp")).exists());
    }

    // ------------------------------------------------------------- T005

    #[test]
    fn config_view_nunca_expoe_o_token_completo() {
        let mut file = SyncConfigFile::default();
        file.token = Some("ghp_1234567890abcd".into());
        file.project_mut("horizon").repo = "a/b".into();

        let view = config_view("horizon", &file).unwrap();
        assert!(view.has_token);
        assert_eq!(view.token_hint.as_deref(), Some("abcd"));
        // O valor completo não aparece em nenhuma parte da view serializada.
        let json = serde_json::to_string(&view).unwrap();
        assert!(!json.contains("ghp_1234567890abcd"));
    }

    #[test]
    fn config_view_de_projeto_vazio_nao_e_erro() {
        let file = SyncConfigFile::default();
        let view = config_view("src", &file).unwrap();
        assert_eq!(view.repo, "");
        assert!(!view.has_token);
        assert_eq!(config_view("outro", &file).unwrap_err().kind(), "sync_config");
    }

    #[test]
    fn set_project_config_valida_e_preserva_o_resto_do_arquivo() {
        let d = dir();
        let mut file = SyncConfigFile::default();
        file.token = Some("ghp_token".into());
        file.project_mut("src").repo = "a/src".into();
        file.project_mut("src").branch = "main".into();
        file.project_mut("src").path = "dados/src.json".into();
        save_config(d.path(), &file).unwrap();

        let saved = set_project_config(
            d.path(),
            "horizon",
            "rafaeldeps/sync",
            "main",
            "exports/exports_canonical.zip",
        )
        .unwrap();

        // Token e SRC preservados; horizon gravado.
        assert_eq!(saved.token.as_deref(), Some("ghp_token"));
        assert_eq!(saved.project("src").unwrap().repo, "a/src");
        assert_eq!(saved.project("horizon").unwrap().path, "exports/exports_canonical.zip");
        // Persistiu de fato.
        assert_eq!(load_config(d.path()).unwrap().project("horizon").unwrap().branch, "main");

        // Destino inválido → sync_config, nada gravado.
        let err = set_project_config(d.path(), "horizon", "sem-barra", "main", "a.zip").unwrap_err();
        assert_eq!(err.kind(), "sync_config");
    }

    #[test]
    fn set_project_config_preserva_last_source_url() {
        let d = dir();
        let mut file = SyncConfigFile::default();
        file.project_mut("horizon").last_source_url =
            Some("https://raw.githubusercontent.com/a/b/main/x.zip".into());
        save_config(d.path(), &file).unwrap();

        let saved = set_project_config(d.path(), "horizon", "a/b", "main", "x.zip").unwrap();
        assert_eq!(
            saved.project("horizon").unwrap().last_source_url.as_deref(),
            Some("https://raw.githubusercontent.com/a/b/main/x.zip")
        );
    }

    #[cfg(unix)]
    #[test]
    fn config_tem_permissao_restrita_0600() {
        use std::os::unix::fs::PermissionsExt;
        let d = dir();
        save_config(d.path(), &SyncConfigFile::default()).unwrap();
        let mode = std::fs::metadata(config_path(d.path())).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn config_ilegivel_da_erro_de_config() {
        let d = dir();
        std::fs::write(config_path(d.path()), "{isso não é json").unwrap();
        let err = load_config(d.path()).unwrap_err();
        assert_eq!(err.kind(), "sync_config");
    }

    #[test]
    fn token_hint_mostra_so_os_ultimos_4() {
        assert_eq!(
            token_hint(&Some("ghp_abcdefghij".into())).unwrap(),
            "ghij"
        );
        assert!(token_hint(&None).is_none());
    }

    // -------------------------------------------------------- validações

    #[test]
    fn repo_valida_formato_owner_name() {
        assert!(validate_repo("rafaeldeps/research-sync").is_ok());
        assert_eq!(validate_repo("so-owner").unwrap_err().kind(), "sync_config");
        assert_eq!(validate_repo("a b/c").unwrap_err().kind(), "sync_config");
        assert_eq!(validate_repo("a//").unwrap_err().kind(), "sync_config");
    }

    #[test]
    fn branch_valida_regras() {
        assert!(validate_branch("main").is_ok());
        assert!(validate_branch("release/v2").is_ok());
        assert_eq!(validate_branch("").unwrap_err().kind(), "sync_config");
        assert_eq!(validate_branch("duas palavras").unwrap_err().kind(), "sync_config");
        assert_eq!(validate_branch("-flag").unwrap_err().kind(), "sync_config");
    }

    #[test]
    fn path_valida_regras_e_zip_no_horizon() {
        assert!(validate_path("horizon", "exports/exports_canonical.zip").is_ok());
        assert_eq!(
            validate_path("horizon", "exports/consolidado.json").unwrap_err().kind(),
            "sync_config"
        );
        assert_eq!(
            validate_path("horizon", "a/../b.zip").unwrap_err().kind(),
            "sync_config"
        );
        // SRC exporta JSON local; o path aqui é de download livre.
        assert!(validate_path("src", "dados/consolidado.json").is_ok());
        assert_eq!(validate_path("src", "").unwrap_err().kind(), "sync_config");
    }

    // ------------------------------------------------------------ política

    #[test]
    fn envio_do_src_e_bloqueado_por_politica_e_horizon_livre() {
        assert!(check_upload_allowed("horizon").is_ok());
        let err = check_upload_allowed("src").unwrap_err();
        assert_eq!(err.kind(), "sync_policy");
        assert!(err.to_string().contains("dados pessoais"));
        assert_eq!(
            check_upload_allowed("outro").unwrap_err().kind(),
            "sync_config"
        );
    }

    // --------------------------------------------------------------- URLs

    #[test]
    fn normaliza_url_raw() {
        let src = normalize_url(
            "https://raw.githubusercontent.com/rafaeldeps/research-sync/main/exports/exports_canonical.zip",
        )
        .unwrap();
        assert_eq!(
            src,
            NormalizedSource::File {
                owner: "rafaeldeps".into(),
                repo: "research-sync".into(),
                reference: "main".into(),
                path: "exports/exports_canonical.zip".into(),
            }
        );
        assert_eq!(src.file_name(), "exports_canonical.zip");
    }

    #[test]
    fn normaliza_url_blob() {
        let src = normalize_url(
            "https://github.com/rafaeldeps/research-sync/blob/main/dados/src_consolidado.json",
        )
        .unwrap();
        assert_eq!(
            src,
            NormalizedSource::File {
                owner: "rafaeldeps".into(),
                repo: "research-sync".into(),
                reference: "main".into(),
                path: "dados/src_consolidado.json".into(),
            }
        );
    }

    #[test]
    fn normaliza_url_release_asset() {
        let src = normalize_url(
            "https://github.com/rafaeldeps/research-sync/releases/download/v1.2.0/exports_canonical.zip",
        )
        .unwrap();
        assert_eq!(
            src,
            NormalizedSource::ReleaseAsset {
                owner: "rafaeldeps".into(),
                repo: "research-sync".into(),
                tag: "v1.2.0".into(),
                asset: "exports_canonical.zip".into(),
            }
        );
        assert_eq!(src.file_name(), "exports_canonical.zip");
    }

    #[test]
    fn rejeita_urls_fora_do_github_e_malformadas() {
        assert_eq!(
            normalize_url("https://gitlab.com/a/b/-/raw/main/x.zip").unwrap_err().kind(),
            "validation"
        );
        assert_eq!(
            normalize_url("https://github.com/rafaeldeps/research-sync").unwrap_err().kind(),
            "validation"
        );
        assert_eq!(normalize_url("não é url").unwrap_err().kind(), "validation");
        assert_eq!(
            normalize_url("http://github.com/a/b/blob/main/x.zip").unwrap_err().kind(),
            "validation"
        );
    }

    #[test]
    fn git_blob_sha_calcula_hash_canonico() {
        assert_eq!(
            git_blob_sha(b"hello world\n"),
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad"
        );
    }
}

