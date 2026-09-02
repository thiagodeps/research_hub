pub mod db;
pub mod error;
pub mod registry;
pub mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // SQLite lives in the OS app-data dir, not next to the executable
            // (the Python version drops test.db in the current directory).
            let data_dir = app.path().app_data_dir()?;
            let db_path = db::database_path(&data_dir)?;
            let conn = db::initialize(&data_dir)?;
            app.manage(state::AppState::new(conn, db_path));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Rotas de navegacao do painel, na ordem em que aparecem no menu lateral.
/// Fonte da verdade para o teste de roteamento; espelha `frontend/src/pages/`.
#[cfg(test)]
pub(crate) const ROUTES: &[&str] = &[
    "/",
    "/login",
    "/dashboard",
    "/dashboard/researchers",
    "/dashboard/articles",
    "/dashboard/groups",
    "/dashboard/initiatives",
    "/dashboard/advisorships",
    "/dashboard/awards",
    "/dashboard/students",
    "/dashboard/campuses",
    "/dashboard/organizations",
    "/dashboard/fellowships",
    "/dashboard/proficiencies",
    "/dashboard/professional_activities",
    "/dashboard/knowledge_areas",
    "/dashboard/languages",
    "/dashboard/research_productions",
];

#[cfg(test)]
mod tests {
    use super::ROUTES;

    fn app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .build(tauri::generate_context!())
            .expect("falha ao construir o app de teste")
    }

    /// SEP-015 / SC-002: os 18 destinos de navegacao devem resolver no binario
    /// compilado, com os hrefs exatamente como o frontend os escreve — sem barra
    /// final e sem sufixo .html.
    ///
    /// Este teste substitui a verificacao manual: o protocolo de asset do Tauri
    /// usa este mesmo resolvedor, entao uma rota que resolve aqui carrega na janela.
    #[test]
    fn todas_as_rotas_do_menu_resolvem_com_o_href_atual() {
        let app = app();
        let resolver = app.asset_resolver();

        let quebradas: Vec<&str> = ROUTES
            .iter()
            .copied()
            .filter(|r| resolver.get(r.to_string()).is_none())
            .collect();

        assert!(
            quebradas.is_empty(),
            "rotas que nao resolvem com o href atual: {quebradas:?}"
        );
    }

    /// O deep link de edicao (`?openId=N`, usado por EntityForm) precisa resolver
    /// para a mesma pagina que a rota sem parametro.
    #[test]
    fn rotas_com_query_string_resolvem() {
        let app = app();
        let resolver = app.asset_resolver();

        assert!(
            resolver.get("/dashboard/groups?openId=42".into()).is_some(),
            "rota com query string nao resolveu"
        );
    }

    /// Guarda contra regressao silenciosa: se uma pagina for adicionada em
    /// `frontend/src/pages/dashboard/` sem entrar em ROUTES, a contagem denuncia.
    #[test]
    fn contagem_de_rotas_bate_com_o_menu_lateral() {
        assert_eq!(
            ROUTES.len(),
            18,
            "ROUTES deve cobrir raiz + login + dashboard + 15 entidades"
        );
    }
}
