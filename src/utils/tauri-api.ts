import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import i18n from '../i18n';

export function translateBackendString(text: string): string {
  if (!text) return text;

  const isI18nKey = text.startsWith('health.') || text.startsWith('logParser.') || text.startsWith('app.');
  if (!isI18nKey) return text;

  const parts = text.split('|');
  const key = `app.${parts[0]}`;

  if (parts.length === 1) {
    const result = i18n.t(key);
    return result !== key ? result : text;
  }

  if (parts[0] === 'logParser.foundIssues' && parts.length >= 3) {
    return i18n.t(key, { errorCount: parts[1], warningCount: parts[2] });
  }

  if (parts[0] === 'logParser.missingDep' || parts[0] === 'logParser.failedToLoad' ||
      parts[0] === 'logParser.incompatible' || parts[0] === 'logParser.runtimeException' ||
      parts[0] === 'logParser.fileNotFound' || parts[0] === 'logParser.modConflict' ||
      parts[0] === 'logParser.obsoleteApi' || parts[0] === 'logParser.updateAvailable') {
    return i18n.t(key, { modName: parts[1] });
  }

  if (parts[0] === 'logParser.missingDepSolution' || parts[0] === 'logParser.failedToLoadSolution' ||
      parts[0] === 'logParser.incompatibleSolution' || parts[0] === 'logParser.runtimeExceptionSolution' ||
      parts[0] === 'logParser.fileNotFoundSolution' || parts[0] === 'logParser.modConflictSolution' ||
      parts[0] === 'logParser.obsoleteApiSuggestion' || parts[0] === 'logParser.updateAvailableSuggestion') {
    return i18n.t(key, { modName: parts[1] });
  }

  if (parts[0] === 'logParser.logFileNotExist' || parts[0] === 'logParser.logReadFailed') {
    return i18n.t(key, { path: parts[1] });
  }

  const result = i18n.t(key);
  return result !== key ? result : text;
}

export function toAssetUrl(filePath: string): Promise<string> {
  return Promise.resolve(convertFileSrc(filePath));
}

export interface SmapiInfo {
  installed: boolean;
  version: string | null;
  game_path: string | null;
  error: string | null;
}

export interface GamePathInfo {
  steam_path: string | null;
  gog_path: string | null;
  xbox_path: string | null;
  detected_path: string | null;
  detection_method: string | null;
}

export interface ModInfo {
  name: string;
  version: string;
  author: string;
  description: string;
  unique_id: string;
  enabled: boolean;
  is_required: boolean;
  has_dependencies: boolean;
  dependency_count: number;
  is_content_pack: boolean;
  content_pack_for: string | null;
  folder_path: string;
  has_conflict: boolean;
  conflict_warning: string | null;
  url: string | null;
  category: string;
  screenshot_path: string | null;
  thumbnail_path: string | null;
  has_update: boolean;
  latest_version: string | null;
  update_url: string | null;
  update_notes: string | null;
  nexus_id: string | null;
  nexus_mod_id: number | null;
  dependencies: ModDependencyItem[];
  manifest_content: string | null;
  sub_mods: ModInfo[];
  is_group: boolean;
}

export interface ModDependencyItem {
  unique_id: string;
  minimum_version: string | null;
  is_required: boolean;
}

export interface InstallResult {
  success: boolean;
  mod_name: string | null;
  message: string;
}

export async function detectGamePath(): Promise<GamePathInfo> {
  return invoke<GamePathInfo>('detect_game_path');
}

export async function checkSmapiStatus(customPath?: string): Promise<SmapiInfo> {
  return invoke<SmapiInfo>('check_smapi_status', {
    customPath: customPath || null,
  });
}

export async function setCustomGamePath(path: string): Promise<GamePathInfo> {
  return invoke<GamePathInfo>('set_custom_game_path', { path });
}

export async function scanMods(gamePath?: string): Promise<ModInfo[]> {
  return invoke<ModInfo[]>('scan_mods', {
    gamePath: gamePath || null,
  });
}

export async function toggleModEnabled(modPath: string, enabled: boolean, extraPaths?: string[]): Promise<boolean> {
  return invoke<boolean>('toggle_mod_enabled', { modPath, enabled, extraPaths: extraPaths || null });
}

export async function updateProfileMods(
  gamePath: string,
  profileName: string,
  enabledModIds: string[]
): Promise<ProfileData> {
  return invoke<ProfileData>('profile_update_mods', { gamePath, profileName, enabledModIds });
}

export async function scanProfileMods(gamePath: string): Promise<ProfileModInfo[]> {
  return invoke<ProfileModInfo[]>('profile_scan_mods', { gamePath });
}

export async function launchGame(gamePath: string): Promise<LaunchResult> {
  return invoke<LaunchResult>('launch_game', { gamePath });
}

export interface SmapiInstallResult {
  success: boolean;
  message: string;
}

export async function openSmapiInstaller(): Promise<boolean> {
  return invoke<boolean>('open_smapi_installer');
}

export async function installSmapiLocal(
  zipPath: string,
  gamePath: string,
): Promise<SmapiInstallResult> {
  return invoke<SmapiInstallResult>('install_smapi_local', { zipPath, gamePath });
}

export async function installModFromArchive(
  archivePath: string,
  modsPath: string
): Promise<InstallResult> {
  return invoke<InstallResult>('install_mod_from_archive', {
    archivePath,
    modsPath,
  });
}

export async function installMod(
  archivePath: string,
  modsPath: string
): Promise<InstallResult> {
  return invoke<InstallResult>('install_mod', {
    archivePath,
    modsPath,
  });
}

export interface ModDependencyCheck {
  mod_name: string;
  unique_id: string;
  version: string;
  missing_dependencies: MissingDepInfo[];
  can_install: boolean;
}

export interface MissingDepInfo {
  unique_id: string;
  minimum_version: string | null;
  is_required: boolean;
}

export async function checkModDependencies(
  archivePath: string,
  modsPath: string
): Promise<ModDependencyCheck> {
  return invoke<ModDependencyCheck>('check_mod_dependencies', {
    archivePath,
    modsPath,
  });
}

export async function installModFromFolder(
  sourcePath: string,
  modsPath: string
): Promise<InstallResult> {
  return invoke<InstallResult>('install_mod_from_folder', {
    sourcePath,
    modsPath,
  });
}

export async function uninstallMod(modPath: string): Promise<InstallResult> {
  return invoke<InstallResult>('uninstall_mod', { modPath });
}

export interface LogError {
  raw_message: string;
  translated_message: string;
  severity: string;
  solution: string;
  solution_button_text: string;
}

export interface LogWarning {
  raw_message: string;
  translated_message: string;
  suggestion: string;
}

export interface LogAnalysis {
  errors: LogError[];
  warnings: LogWarning[];
  error_count: number;
  warning_count: number;
  summary: string;
}

export interface SyncModEntry {
  name: string;
  unique_id: string;
  version: string;
  author: string;
  url: string | null;
  enabled: boolean;
}

export interface VersionMismatch {
  mod_entry: SyncModEntry;
  current_version: string;
  required_version: string;
}

export interface ConfigDiff {
  mod_name: string;
  config_file: string;
  status: string;
}

export interface SyncDiff {
  missing_mods: SyncModEntry[];
  version_mismatch: VersionMismatch[];
  extra_mods: SyncModEntry[];
  config_diffs: ConfigDiff[];
  total_changes: number;
  summary: string;
}

export interface SyncApplyResult {
  success: boolean;
  applied_mods: string[];
  failed_mods: string[];
  configs_applied: string[];
  message: string;
}

export interface ProfileSyncDiff {
  local_missing: SyncModEntry[];
  remote_missing: SyncModEntry[];
  version_mismatch: VersionMismatch[];
  common_mods: SyncModEntry[];
  total_changes: number;
  summary: string;
}

export async function analyzeLog(logPath?: string): Promise<LogAnalysis> {
  return invoke<LogAnalysis>('analyze_log', {
    logPath: logPath || null,
  });
}

export interface ParsedLogError {
  mod_name: string;
  error_type: string;
  raw_line: string;
  solution: string;
  severity: string;
}

export interface ParseSmapiLogResult {
  errors: ParsedLogError[];
  log_path: string;
  has_errors: boolean;
  log_not_found: boolean;
  smapi_not_installed: boolean;
}

export async function parseSmapiLog(logPath?: string): Promise<ParseSmapiLogResult> {
  return invoke<ParseSmapiLogResult>('parse_smapi_log', {
    logPath: logPath || null,
  });
}

export async function readLogFile(filePath: string): Promise<string> {
  return invoke<string>('read_log_file', { filePath });
}

export interface FtmLogEntry {
  raw_line: string;
  line_number: number;
}

export interface FtmLogAnalysis {
  log_path: string;
  error_lines: FtmLogEntry[];
  error_count: number;
  core_reason: string;
  plain_explanation: string;
  suggested_action: string;
  has_ftm_errors: boolean;
}

export async function analyzeFtmErrors(): Promise<FtmLogAnalysis> {
  return invoke<FtmLogAnalysis>('analyze_ftm_errors');
}

export async function exportSyncEnvironment(
  gamePath: string,
  hostName: string,
  exportPath: string
): Promise<string> {
  return invoke<string>('export_sync_environment', {
    gamePath,
    hostName,
    exportPath,
  });
}

export async function importSyncEnvironment(
  importPath: string,
  gamePath: string
): Promise<SyncDiff> {
  return invoke<SyncDiff>('import_sync_environment', {
    importPath,
    gamePath,
  });
}

export async function applySyncEnvironment(
  syncPackagePath: string,
  gamePath: string
): Promise<SyncApplyResult> {
  return invoke<SyncApplyResult>('apply_sync_environment', {
    syncPackagePath,
    gamePath,
  });
}

export async function openSaveDialog(): Promise<string | null> {
  return invoke<string | null>('open_save_dialog');
}

export async function openOpenDialog(): Promise<string | null> {
  return invoke<string | null>('open_open_dialog');
}

export async function exportSyncPackage(
  gamePath: string,
  profileName: string,
  hostName: string
): Promise<string> {
  return invoke<string>('export_sync_package', {
    gamePath,
    profileName,
    hostName,
  });
}

export async function compareSyncDiff(
  syncFilePath: string,
  gamePath: string,
  profileName: string
): Promise<ProfileSyncDiff> {
  return invoke<ProfileSyncDiff>('compare_sync_diff', {
    syncFilePath,
    gamePath,
    profileName,
  });
}

export interface NetworkDiagnosticResult {
  target: string;
  reachable: boolean;
  response_time_ms: number | null;
  error: string | null;
}

export async function diagnoseNetwork(): Promise<NetworkDiagnosticResult[]> {
  return invoke<NetworkDiagnosticResult[]>('diagnose_network');
}

export async function testNexusConnection(): Promise<NetworkDiagnosticResult> {
  return invoke<NetworkDiagnosticResult>('test_nexus_connection');
}

export interface SaveInfo {
  name: string;
  farm_name: string;
  farm_type: string;
  hours_played: number;
  last_modified: string;
  save_path: string;
  backup_count: number;
  linked_profile: string | null;
  character_name: string;
}

export interface BackupInfo {
  name: string;
  original_name: string;
  backup_time: string;
  backup_path: string;
  size_mb: number;
}

export interface SaveBackupResult {
  success: boolean;
  backup_path: string;
  message: string;
}

export interface SaveRestoreResult {
  success: boolean;
  message: string;
}

export async function scanSaves(): Promise<SaveInfo[]> {
  return invoke<SaveInfo[]>('scan_saves');
}

export async function backupSave(savePath: string, backupDir: string): Promise<SaveBackupResult> {
  return invoke<SaveBackupResult>('backup_save', { savePath, backupDir });
}

export async function restoreSave(backupPath: string, savesDir: string): Promise<SaveRestoreResult> {
  return invoke<SaveRestoreResult>('restore_save', { backupPath, savesDir });
}

export async function listSaveBackups(savePath: string): Promise<BackupInfo[]> {
  return invoke<BackupInfo[]>('list_save_backups', { savePath });
}

export async function linkSaveToProfile(savePath: string, profileName: string): Promise<boolean> {
  return invoke<boolean>('link_save_to_profile', { savePath, profileName });
}

export async function unlinkSaveFromProfile(savePath: string): Promise<boolean> {
  return invoke<boolean>('unlink_save_from_profile', { savePath });
}

export async function getSaveProfileBinding(savePath: string): Promise<string | null> {
  return invoke<string | null>('get_save_profile_binding', { savePath });
}

export interface LaunchResult {
  success: boolean;
  message: string;
}

export async function launchGameWithSaveProfile(gamePath: string, savePath: string): Promise<LaunchResult> {
  return invoke<LaunchResult>('launch_game_with_save_profile', { gamePath, savePath });
}

export async function openSaveLocation(): Promise<boolean> {
  return invoke<boolean>('open_save_location');
}

export async function openBackupDialog(): Promise<string> {
  return invoke<string>('open_backup_dialog');
}

export interface GameSessionInfo {
  is_running: boolean;
  pid: number | null;
  start_time: string | null;
}

export async function getGameSessionInfo(): Promise<GameSessionInfo> {
  return invoke<GameSessionInfo>('get_game_session_info');
}

export async function restoreSvlWindow(): Promise<boolean> {
  return invoke<boolean>('restore_svl_window');
}

export async function toggleMod(modPath: string, enabled: boolean, extraPaths?: string[]): Promise<boolean> {
  return invoke<boolean>('toggle_mod_enabled', { modPath, enabled, extraPaths: extraPaths || null });
}

export async function deleteMod(modPath: string): Promise<InstallResult> {
  return invoke<InstallResult>('uninstall_mod', { modPath });
}

export interface NexusApiVerification {
  success: boolean;
  is_premium: boolean;
  user_name: string | null;
  message: string | null;
}

export interface NxmLinkInfo {
  game_id: string;
  mod_id: string;
  file_id: string;
  original_url: string;
}

export interface ModUpdateInfo {
  unique_id: string;
  name: string;
  current_version: string;
  latest_version: string | null;
  nexus_mod_id: string | null;
  has_update: boolean;
  download_url: string | null;
}

export interface NexusFileDownloadInfo {
  file_id: string;
  name: string;
  version: string;
  size: number;
  upload_time: string;
  download_url: string | null;
  is_premium_only: boolean;
  category_id: number;
}

export async function verifyNexusApiKey(apiKey: string): Promise<NexusApiVerification> {
  return invoke<NexusApiVerification>('verify_nexus_api_key', { apiKey });
}

export async function parseNxmLink(nxmUrl: string): Promise<NxmLinkInfo> {
  return invoke<NxmLinkInfo>('parse_nxm_link', { nxmUrl });
}

export async function checkModUpdates(
  apiKey: string,
  modsData: any[]
): Promise<ModUpdateInfo[]> {
  return invoke<ModUpdateInfo[]>('check_mod_updates', { apiKey, modsData });
}

export interface NxmProtocolResult {
  success: boolean;
  message: string;
}

export async function endorseMod(apiKey: string, modId: string): Promise<boolean> {
  return invoke<boolean>('endorse_mod', { apiKey, modId });
}

export async function getNexusModFiles(
  apiKey: string,
  modId: string
): Promise<NexusFileDownloadInfo[]> {
  return invoke<NexusFileDownloadInfo[]>('get_nexus_mod_files', { apiKey, modId });
}

export async function getNexusDownloadUrl(uniqueId: string, apiKey: string = '', modFolderPath?: string): Promise<string> {
  return invoke<string>('get_nexus_download_url', { uniqueId, apiKey, modFolderPath: modFolderPath || null });
}

export async function handleNxmLink(nxmUrl: string): Promise<NxmLinkInfo> {
  return invoke<NxmLinkInfo>('handle_nxm_link', { nxmUrl });
}

export async function registerNxmProtocol(): Promise<NxmProtocolResult> {
  return invoke<NxmProtocolResult>('register_nxm_protocol');
}

export interface ModDownloadResult {
  success: boolean;
  mod_name: string;
  mod_version: string;
  message: string;
  file_size: number;
}

export async function downloadModFromNexus(
  modId: string,
  apiKey: string,
  modsPath?: string | null,
  fileId?: string
): Promise<ModDownloadResult> {
  return invoke<ModDownloadResult>('download_mod_from_nexus', {
    modId,
    apiKey,
    modsPath: modsPath || null,
    fileId: fileId || null,
  });
}

export interface ModDictUpdateResult {
  success: boolean;
  new_entries: number;
  total_entries: number;
  message: string;
}

export async function updateModDict(): Promise<ModDictUpdateResult> {
  return invoke<ModDictUpdateResult>('update_mod_dict');
}

export interface ProfileData {
  name: string;
  is_protected: boolean;
  enabled_mod_ids: string[];
  created_at: string;
  last_used: string;
}

export interface ProfileListItem {
  name: string;
  is_protected: boolean;
  is_active: boolean;
  total_mods: number;
  enabled_count: number;
  created_at: string;
  last_used: string;
}

export interface ProfileModInfo {
  unique_id: string;
  name: string;
  version: string;
  author: string;
  is_required: boolean;
}

export async function profileCreate(
  gamePath: string,
  profileName: string,
  enabledModIds?: string[]
): Promise<ProfileData> {
  return invoke<ProfileData>('profile_create', { gamePath, profileName, enabledModIds: enabledModIds || null });
}

export async function profileList(gamePath: string): Promise<ProfileListItem[]> {
  return invoke<ProfileListItem[]>('profile_list', { gamePath });
}

export async function profileGetActive(gamePath: string): Promise<string | null> {
  return invoke<string | null>('profile_get_active', { gamePath });
}

export async function profileSwitch(gamePath: string, profileName: string): Promise<ProfileData> {
  return invoke<ProfileData>('profile_switch', { gamePath, profileName });
}

export async function profileDelete(gamePath: string, profileName: string): Promise<boolean> {
  return invoke<boolean>('profile_delete', { gamePath, profileName });
}

export async function profileToggleMod(
  gamePath: string,
  profileName: string,
  modId: string,
  enabled: boolean
): Promise<ProfileData> {
  return invoke<ProfileData>('profile_toggle_mod', { gamePath, profileName, modId, enabled });
}

export async function profileGetModStates(
  gamePath: string,
  profileName: string
): Promise<Record<string, boolean>> {
  return invoke<Record<string, boolean>>('profile_get_mod_states', { gamePath, profileName });
}

export async function profileClearActive(gamePath: string): Promise<boolean> {
  return invoke<boolean>('profile_clear_active', { gamePath });
}

export async function profileCopy(
  gamePath: string,
  fromProfile: string,
  newProfileName: string
): Promise<ProfileData> {
  return invoke<ProfileData>('profile_copy', { gamePath, fromProfile, newProfileName });
}

export async function profileExport(
  gamePath: string,
  profileName: string,
  exportPath: string
): Promise<boolean> {
  return invoke<boolean>('profile_export', { gamePath, profileName, exportPath });
}

export async function profileImport(
  gamePath: string,
  importPath: string
): Promise<ProfileData> {
  return invoke<ProfileData>('profile_import', { gamePath, importPath });
}

export async function getProfileBindings(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('get_profile_bindings');
}

export async function setProfileBinding(
  saveFolderName: string,
  profileName: string | null
): Promise<boolean> {
  return invoke<boolean>('set_profile_binding', { saveFolderName, profileName });
}

export interface ModUpdateStatus {
  unique_id: string;
  name: string;
  current_version: string;
  latest_version: string | null;
  has_update: boolean;
  update_source: 'SmapiList' | 'NexusApi' | 'UnofficialUpdate' | 'None';
  download_url: string | null;
  nexus_mod_id: string | null;
  changelog: string | null;
  is_nexus_premium: boolean;
}

export interface BatchUpdateResult {
  total: number;
  updated: number;
  failed: number;
  details: Array<{
    unique_id: string;
    name: string;
    success: boolean;
    message: string;
  }>;
}

export async function checkSingleModUpdate(
  uniqueId: string,
  currentVersion: string,
  modFolderPath?: string
): Promise<ModUpdateStatus> {
  return invoke<ModUpdateStatus>('check_single_mod_update', {
    uniqueId,
    currentVersion,
    modFolderPath: modFolderPath || null,
  });
}

export async function checkAllModsUpdates(
  modsData: any[],
  apiKey?: string
): Promise<ModUpdateStatus[]> {
  return invoke<ModUpdateStatus[]>('check_all_mods_updates', {
    modsData,
    apiKey: apiKey || null,
  });
}

export async function batchUpdateMods(
  modsToUpdate: any[],
  apiKey: string,
  modsPath: string,
): Promise<BatchUpdateResult> {
  return invoke<BatchUpdateResult>('batch_update_mods', {
    modsToUpdate,
    apiKey,
    modsPath,
  });
}

export async function downloadModUpdate(
  nexusModId: string,
  apiKey: string,
  modsPath: string,
  oldUniqueId?: string,
): Promise<string> {
  return invoke<string>('download_mod_update', {
    nexusModId,
    apiKey,
    modsPath,
    oldUniqueId: oldUniqueId || null,
  });
}

export interface ProfileExportResult {
  success: boolean;
  zip_path: string;
  mod_count: number;
  message: string;
}

export interface ModpackImportResult {
  success: boolean;
  profile_name: string;
  mod_count: number;
  message: string;
}

export async function exportProfileToZip(
  profileName: string,
  gamePath: string
): Promise<ProfileExportResult> {
  return invoke<ProfileExportResult>('export_profile_to_zip', { profileName, gamePath });
}

export async function importModpackFromZip(
  zipPath: string,
  targetProfileName: string,
  gamePath: string
): Promise<ModpackImportResult> {
  return invoke<ModpackImportResult>('import_modpack_from_zip', { zipPath, targetProfileName, gamePath });
}

export async function importModpackFromFolder(
  folderPath: string,
  targetProfileName: string,
  gamePath: string
): Promise<ModpackImportResult> {
  return invoke<ModpackImportResult>('import_modpack_from_folder', { folderPath, targetProfileName, gamePath });
}

export interface AppUpdateInfo {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  download_url: string;
  release_notes: string | null;
  release_date: string | null;
  file_size: number | null;
  sha256: string | null;
  force_update: boolean;
  source: string | null;
}

export interface AppUpdateProgress {
  downloaded: number;
  total: number;
  percent: number;
}

export interface ModBackupResult {
  success: boolean;
  backup_path: string;
  message: string;
}

export async function backupModBeforeUpdate(
  modPath: string,
  customBackupDir?: string
): Promise<ModBackupResult> {
  return invoke<ModBackupResult>('backup_mod_before_update', {
    modPath,
    customBackupDir,
  });
}

export interface AppUpdateResult {
  success: boolean;
  message: string;
  needs_restart: boolean;
  file_path?: string;
}

export async function checkAppUpdateFromServer(): Promise<AppUpdateInfo> {
  return invoke<AppUpdateInfo>('check_app_update_from_server');
}

export async function checkAppUpdateGithub(): Promise<AppUpdateInfo> {
  return invoke<AppUpdateInfo>('check_app_update_github');
}

export async function downloadAppUpdateFromServer(downloadUrl: string): Promise<AppUpdateResult> {
  return invoke<AppUpdateResult>('download_app_update_from_server', { downloadUrl });
}

export async function runInstaller(path: string): Promise<void> {
  return invoke<void>('run_installer', { path });
}

export async function getUpdateServerUrl(): Promise<string> {
  return invoke<string>('get_update_server_url');
}

export async function getCurrentAppVersion(): Promise<string> {
  return invoke<string>('get_current_app_version');
}

export interface ThumbnailCacheInfo {
  size: number;
  sizeFormatted: string;
  fileCount: number;
  cachePath: string;
}

export async function refreshModThumbnail(
  modUniqueId: string,
  nexusModId: number
): Promise<string | null> {
  return invoke<string | null>('refresh_mod_thumbnail', { modUniqueId, nexusModId });
}

export async function clearThumbnailCache(): Promise<number> {
  return invoke<number>('clear_thumbnail_cache');
}

export async function getThumbnailCacheInfo(): Promise<ThumbnailCacheInfo> {
  return invoke<ThumbnailCacheInfo>('get_thumbnail_cache_info');
}

export interface MissingDependency {
  unique_id: string;
  display_name: string;
  is_required: boolean;
  minimum_version: string | null;
  nexus_mod_id: string | null;
  nexus_url: string;
  required_by: string[];
}

export interface DependencyScanResult {
  total_installed: number;
  total_missing: number;
  missing_dependencies: MissingDependency[];
}

export interface DependencyInstallResult {
  success: boolean;
  mod_name: string;
  message: string;
}

export async function scanAllMissingDependencies(
  modsPath: string
): Promise<DependencyScanResult> {
  return invoke<DependencyScanResult>('scan_all_missing_dependencies', { modsPath });
}

export async function autoInstallMissingDependency(
  uniqueId: string,
  nexusModId: string | null,
  modsPath: string,
  apiKey: string
): Promise<DependencyInstallResult> {
  return invoke<DependencyInstallResult>('auto_install_missing_dependency', {
    uniqueId,
    nexusModId,
    modsPath,
    apiKey,
  });
}

export interface ConflictReport {
  mod_name: string;
  unique_id: string;
  conflict_type: 'MissingDependency' | 'OptionalDependencyMissing' | 'ContentPackConflict' | 'Incompatibility' | 'HardcodedPatch' | 'AssetConflict' | 'ContentPackTargetConflict' | 'VersionConflict';
  description: string;
  severity: 'Error' | 'Warning' | 'Info';
  solution: string;
  affected_mods: string[] | null;
}

export async function checkConflicts(mods: ModInfo[]): Promise<ConflictReport[]> {
  return invoke<ConflictReport[]>('check_conflicts', { mods });
}

export interface ModStorageInfo {
  name: string;
  unique_id: string;
  folder_path: string;
  size_bytes: number;
  size_formatted: string;
  file_count: number;
  enabled: boolean;
  is_content_pack: boolean;
  version: string;
}

export interface StorageAnalysisResult {
  mods: ModStorageInfo[];
  total_size_bytes: number;
  total_size_formatted: string;
  total_mods: number;
  enabled_size_bytes: number;
  enabled_size_formatted: string;
  disabled_size_bytes: number;
  disabled_size_formatted: string;
  largest_mod: ModStorageInfo | null;
}

export async function analyzeModStorage(mods: ModInfo[]): Promise<StorageAnalysisResult> {
  return invoke<StorageAnalysisResult>('analyze_mod_storage', { mods });
}

export interface ModConfigListItem {
  mod_name: string;
  unique_id: string;
  folder_path: string;
  config_path: string;
  field_count: number;
  has_config: boolean;
}

export interface ModConfigListResult {
  configs: ModConfigListItem[];
  total_mods_with_config: number;
  total_mods_scanned: number;
}

export async function listModConfigs(modsPath: string): Promise<ModConfigListResult> {
  return invoke<ModConfigListResult>('list_mod_configs', { modsPath });
}

export async function readModConfig(modPath: string): Promise<any> {
  return invoke<any>('read_mod_config', { modPath });
}

export async function updateModConfig(modPath: string, updates: Array<{ key: string; value: any }>): Promise<{ success: boolean; message: string }> {
  return invoke<{ success: boolean; message: string }>('update_mod_config', { modPath, updates });
}

export interface ModSnapshotInfo {
  snapshot_name: string;
  created_at: string;
  mod_count: number;
  size_mb: number;
  label: string;
  snapshot_path: string;
}

export interface ModSnapshotList {
  snapshots: ModSnapshotInfo[];
  total_snapshots: number;
  total_size_mb: number;
}

export interface SnapshotResult {
  success: boolean;
  message: string;
}

export async function createSnapshot(modsPath: string, label: string): Promise<SnapshotResult> {
  return invoke<SnapshotResult>('create_snapshot', { modsPath, label });
}

export async function listSnapshots(): Promise<ModSnapshotList> {
  return invoke<ModSnapshotList>('list_snapshots');
}

export async function restoreSnapshot(snapshotName: string, modsPath: string): Promise<SnapshotResult> {
  return invoke<SnapshotResult>('restore_snapshot', { snapshotName, modsPath });
}

export async function deleteSnapshot(snapshotName: string): Promise<SnapshotResult> {
  return invoke<SnapshotResult>('delete_snapshot', { snapshotName });
}

export interface LogFileInfo {
  name: string;
  path: string;
  size_bytes: number;
  modified: string;
}

export interface AppLogResult {
  lines: string[];
  total_lines: number;
  log_dir: string;
  files: LogFileInfo[];
}

export async function getAppLogs(maxLines?: number): Promise<AppLogResult> {
  return invoke<AppLogResult>('get_app_logs', { maxLines: maxLines || null });
}

export async function exportAppLogs(): Promise<string> {
  return invoke<string>('export_app_logs');
}

export async function clearOldAppLogs(keepDays?: number): Promise<string> {
  return invoke<string>('clear_old_app_logs', { keepDays: keepDays || null });
}

export async function getLogDirPath(): Promise<string> {
  return invoke<string>('get_log_dir_path');
}
