pub mod smapi;
pub mod mod_parser;
mod mod_installer;
pub mod profiles;
mod log_parser;
mod sync_manager;
mod smapi_installer;
mod saves_manager;
mod nexus_api;
mod smapi_launcher;
mod profile_archive;
mod mod_dict_updater;
mod compatibility_list;
mod mod_name_resolver;
mod dependency_patches;
pub mod conflict_checker;
mod update_checker;
mod smapi_data;
mod mod_patches;
mod nexus_linker;
mod mod_thumbnail;
mod mod_ordering;
mod mod_config;
mod mod_backup;
mod mod_security;
mod app_updater;
mod dep_resolver;
mod storage_analyzer;
mod app_logger;
mod mod_translator;
mod mod_name_translator;

use smapi::{detect_game_path, check_smapi_status, set_custom_game_path, open_smapi_installer, restore_svl_window};
use smapi_installer::{install_smapi_local, open_smapi_zip_dialog};
use smapi_launcher::{launch_game, get_game_session_info, stop_game};
use mod_parser::{scan_mods, toggle_mod_enabled};
use mod_installer::{install_mod_from_archive, install_mod_from_folder, uninstall_mod, install_mod, check_mod_dependencies};
use profiles::{profile_create, profile_list, profile_get_active, profile_switch, profile_delete, profile_update_mods, profile_toggle_mod, profile_get_mod_states, profile_clear_active, profile_copy, profile_export, profile_import, profile_scan_mods, get_profile_bindings, set_profile_binding, get_essential_mod_ids};
use log_parser::{analyze_log, parse_smapi_log, read_log_file, check_smapi_log, get_appdata_path, analyze_ftm_errors, open_path, fix_all_log_errors, fix_single_log_error, check_dotnet_status};
use sync_manager::{export_sync_environment, import_sync_environment, apply_sync_environment, open_save_dialog, open_open_dialog, export_sync_package, compare_sync_diff};
use saves_manager::{scan_saves, backup_save, restore_save, list_save_backups, link_save_to_profile, unlink_save_from_profile, get_save_profile_binding, launch_game_with_save_profile, open_save_location, open_backup_dialog};
use nexus_api::{verify_nexus_api_key, parse_nxm_link, handle_nxm_link, register_nxm_protocol, check_mod_updates, endorse_mod, get_nexus_mod_files, get_nexus_download_url, download_mod_from_nexus, search_nexus_mods, get_trending_nexus_mods, get_recently_updated_nexus_mods, get_monthly_top_nexus_mods, browse_nexus_category, get_nexus_categories, open_nexus_browser, close_nexus_browser, download_mod_from_cdn_link, diagnose_network, test_nexus_connection};
use mod_dict_updater::{update_mod_dict, auto_update_mod_dict};
use compatibility_list::{update_compatibility_list, get_compatibility_status, init_compatibility_cache, auto_update_compatibility_list};
use profile_archive::{export_profile_to_zip, import_modpack_from_zip, import_modpack_from_folder};
use conflict_checker::check_conflicts;
use update_checker::{check_single_mod_update, check_all_mods_updates, batch_update_mods, download_mod_update};
use nexus_linker::get_nexus_link;
use mod_thumbnail::{refresh_mod_thumbnail, clear_thumbnail_cache, get_thumbnail_cache_info};
use mod_ordering::{calculate_optimal_load_order, apply_load_order};
use mod_config::{read_mod_config, update_mod_config, list_mod_configs};
use mod_backup::{backup_mod_before_update, restore_mod_from_backup, list_mod_backups, delete_mod_backup, create_snapshot, list_snapshots, restore_snapshot, delete_snapshot};
use mod_security::{start_game_monitor, stop_game_monitor, get_monitor_status, check_mod_security, batch_check_mod_security};
use app_updater::{check_app_update_from_server, check_app_update_github, download_app_update_from_server, get_update_server_url, get_current_app_version, run_installer, auto_check_app_update};
use dep_resolver::{scan_all_missing_dependencies, auto_install_missing_dependency};
use storage_analyzer::analyze_mod_storage;
use app_logger::{get_app_logs, export_app_logs, clear_old_app_logs, get_log_dir_path, log_info, log_warn, log_error};
use mod_translator::{scan_translatable_mods, translate_mod_file, test_ai_connection, restore_translation_backup, scan_translation_backups};
use mod_name_translator::{get_mod_name_translations, translate_mod_name, batch_translate_mod_names, delete_mod_name_translation, clear_all_mod_name_translations};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())

        .setup(|app| {
            let handle = app.handle().clone();
            let update_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    smapi_data::init_smapi_cache().await;
                    init_compatibility_cache().await;
                    auto_update_compatibility_list().await;
                    auto_update_mod_dict().await;
                    auto_check_app_update(update_handle).await;
                });
            });

            mod_security::monitor_game_loop(handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            detect_game_path,
            check_smapi_status,
            set_custom_game_path,
            scan_mods,
            toggle_mod_enabled,
            launch_game,
            open_smapi_installer,
            install_smapi_local,
            open_smapi_zip_dialog,
            install_mod_from_archive,
            install_mod_from_folder,
            uninstall_mod,
            install_mod,
            check_mod_dependencies,
            profile_create,
            profile_list,
            profile_get_active,
            profile_switch,
            profile_delete,
            profile_update_mods,
            profile_toggle_mod,
            profile_get_mod_states,
            profile_clear_active,
            profile_copy,
            profile_export,
            profile_import,
            profile_scan_mods,
            get_profile_bindings,
            set_profile_binding,
            analyze_log,
            parse_smapi_log,
            read_log_file,
            check_smapi_log,
            get_appdata_path,
            analyze_ftm_errors,
            open_path,
            fix_all_log_errors,
            fix_single_log_error,
            check_dotnet_status,
            export_sync_environment,
            import_sync_environment,
            apply_sync_environment,
            open_save_dialog,
            open_open_dialog,
            export_sync_package,
            compare_sync_diff,
            scan_saves,
            backup_save,
            restore_save,
            list_save_backups,
            link_save_to_profile,
            unlink_save_from_profile,
            get_save_profile_binding,
            launch_game_with_save_profile,
            open_save_location,
            open_backup_dialog,
            get_game_session_info,
            stop_game,
            restore_svl_window,
            verify_nexus_api_key,
            parse_nxm_link,
            handle_nxm_link,
            register_nxm_protocol,
            check_mod_updates,
            endorse_mod,
            get_nexus_mod_files,
            get_nexus_download_url,
            download_mod_from_nexus,
            search_nexus_mods,
            get_trending_nexus_mods,
            get_recently_updated_nexus_mods,
            get_monthly_top_nexus_mods,
            browse_nexus_category,
            get_nexus_categories,
            update_mod_dict,
            update_compatibility_list,
            get_compatibility_status,
            export_profile_to_zip,
            import_modpack_from_zip,
            import_modpack_from_folder,
            check_conflicts,
            check_single_mod_update,
            check_all_mods_updates,
            batch_update_mods,
            download_mod_update,
            get_nexus_link,
            refresh_mod_thumbnail,
            clear_thumbnail_cache,
            get_thumbnail_cache_info,
            calculate_optimal_load_order,
            apply_load_order,
            read_mod_config,
            update_mod_config,
            list_mod_configs,
            backup_mod_before_update,
            restore_mod_from_backup,
            list_mod_backups,
            delete_mod_backup,
            create_snapshot,
            list_snapshots,
            restore_snapshot,
            delete_snapshot,
            start_game_monitor,
            stop_game_monitor,
            get_monitor_status,
            check_mod_security,
            batch_check_mod_security,
            get_essential_mod_ids,
            open_nexus_browser,
            close_nexus_browser,
            download_mod_from_cdn_link,
            diagnose_network,
            test_nexus_connection,
            check_app_update_from_server,
            check_app_update_github,
            download_app_update_from_server,
            get_update_server_url,
            get_current_app_version,
            run_installer,
            scan_all_missing_dependencies,
            auto_install_missing_dependency,
            analyze_mod_storage,
            get_app_logs,
            export_app_logs,
            clear_old_app_logs,
            get_log_dir_path,
            scan_translatable_mods,
            translate_mod_file,
            test_ai_connection,
            restore_translation_backup,
            scan_translation_backups,
            get_mod_name_translations,
            translate_mod_name,
            batch_translate_mod_names,
            delete_mod_name_translation,
            clear_all_mod_name_translations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
