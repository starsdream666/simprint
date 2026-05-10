use tauri::ipc::Invoke;

pub mod app;
pub mod auth;
pub mod environment;
pub mod file_system;
pub mod local_api;
pub mod logging;
pub mod mcp;
pub mod network;
pub mod security;
pub mod server;
pub mod store;
pub mod updater;
pub mod window;

pub fn register_handles() -> impl Fn(Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // App commands
        app::complete_and_show_main,
        app::show_main_window,
        app::get_auto_start_state,
        app::set_auto_start_enabled,
        app::set_updating_state,
        app::get_app_state,
        app::splashscreen_ready,
        app::close_program,
        app::get_executable_path,
        app::is_dev,
        // Window commands
        window::get_window_size,
        window::create_syncer_window,
        window::arrange_environments,
        window::calculate_window_layout,
        window::hide_window,
        window::show_window,
        // Environment commands
        environment::get_running_environments,
        environment::start_sync,
        environment::stop_sync,
        environment::export_proxies_to_csv,
        environment::ensure_kernel_ready,
        environment::launch_environment,
        environment::start_environment_by_uuid,
        environment::batch_launch_environments,
        environment::batch_start_environments_by_uuid,
        environment::batch_stop_environments,
        environment::stop_environment,
        environment::refresh_environment_proxy,
        environment::get_connected_environments,
        environment::get_environment_cdp_endpoint,
        environment::get_environment_status,
        environment::get_all_environment_statuses,
        // File system commands
        file_system::read_text_file,
        file_system::write_text_file,
        file_system::get_storage_default_paths,
        file_system::get_directory_sizes,
        // Security commands
        security::get_client_public_key,
        security::report_user_activity,
        security::get_session_lock_state,
        security::unlock_session,
        // Network commands
        network::http_get,
        network::http_post,
        network::http_post_form,
        network::http_put,
        network::http_delete,
        network::test_proxy,
        network::test_direct_ip,
        network::detect_proxy_ip,
        network::download_files,
        // Server endpoint commands
        server::get_server_endpoint_state,
        server::set_server_base_url,
        server::reset_server_base_url,
        // Auth commands
        auth::login,
        auth::register,
        auth::logout,
        auth::save_credential,
        auth::get_access_token,
        auth::is_logged_in,
        auth::save_remembered_credential,
        auth::get_remembered_credential,
        auth::clear_remembered_credential,
        // Updater commands
        updater::check_update_available,
        updater::check_updates,
        updater::download_updates,
        updater::start_update_install,
        updater::start_prepared_update_install,
        updater::get_prepared_update,
        // Core utilities
        crate::core::utils::process::kill_process,
        // Logging commands
        logging::log_info,
        logging::log_error,
        local_api::get_local_api_runtime_running,
        local_api::start_local_api_runtime,
        local_api::reload_local_api_runtime,
        local_api::stop_local_api_runtime,
        mcp::get_mcp_config,
        mcp::update_mcp_config,
        mcp::start_mcp_runtime,
        mcp::reload_mcp_runtime,
        mcp::stop_mcp_runtime,
        // Store commands
        store::get_store_key,
        store::set_store_key,
    ]
}
