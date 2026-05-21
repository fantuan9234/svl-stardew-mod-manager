import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ModInfo } from './tauri-api';

export interface ModLoadOrder {
  unique_id: string;
  name: string;
  position: number;
  layer: string;
  reason: string;
  dependencies: string[];
}

export interface LoadOrderReport {
  ordered_mods: ModLoadOrder[];
  conflicts: string[];
  suggestions: string[];
  total_mods: number;
}

export interface ApplyLoadOrderResult {
  success: boolean;
  message: string;
  moved_count: number;
}

export async function calculateOptimalLoadOrder(mods: ModInfo[]): Promise<LoadOrderReport> {
  return invoke<LoadOrderReport>('calculate_optimal_load_order', { mods });
}

export async function applyLoadOrder(gamePath: string, order: string[]): Promise<ApplyLoadOrderResult> {
  return invoke<ApplyLoadOrderResult>('apply_load_order', { gamePath, order });
}

export interface ConfigField {
  key: string;
  value: ConfigValue;
  field_type: string;
  description: string;
}

export interface ConfigValue {
  type: string;
  value: string | number | boolean | unknown[] | Record<string, unknown> | null;
}

export interface ModConfigSchema {
  mod_name: string;
  unique_id: string;
  config_path: string;
  fields: ConfigField[];
}

export interface UpdateConfigField {
  key: string;
  value: string | number | boolean;
}

export interface UpdateConfigResult {
  success: boolean;
  message: string;
}

export async function readModConfig(modPath: string): Promise<ModConfigSchema> {
  return invoke<ModConfigSchema>('read_mod_config', { modPath });
}

export async function updateModConfig(modPath: string, updates: UpdateConfigField[]): Promise<UpdateConfigResult> {
  return invoke<UpdateConfigResult>('update_mod_config', { modPath, updates });
}

export interface ModBackupInfo {
  backup_name: string;
  mod_name: string;
  mod_unique_id: string;
  backup_path: string;
  created_at: string;
  size_mb: number;
  version: string;
}

export interface ModBackupResult {
  success: boolean;
  backup_path: string;
  message: string;
}

export interface ModRestoreResult {
  success: boolean;
  message: string;
}

export interface ModBackupList {
  backups: ModBackupInfo[];
  total_backups: number;
  total_size_mb: number;
}

export async function backupModBeforeUpdate(modPath: string): Promise<ModBackupResult> {
  return invoke<ModBackupResult>('backup_mod_before_update', { modPath });
}

export async function restoreModFromBackup(backupPath: string, targetModPath: string): Promise<ModRestoreResult> {
  return invoke<ModRestoreResult>('restore_mod_from_backup', { backupPath, targetModPath });
}

export async function listModBackups(modUniqueId?: string): Promise<ModBackupList> {
  return invoke<ModBackupList>('list_mod_backups', { modUniqueId: modUniqueId || null });
}

export async function deleteModBackup(backupPath: string): Promise<ModRestoreResult> {
  return invoke<ModRestoreResult>('delete_mod_backup', { backupPath });
}

export interface ModLoadEvent {
  mod_name: string;
  unique_id: string;
  load_time_ms: number;
  timestamp: string;
}

export interface ModErrorEvent {
  mod_name: string;
  unique_id: string;
  error_message: string;
  severity: string;
  timestamp: string;
}

export interface ModWarningEvent {
  mod_name: string;
  unique_id: string;
  warning_message: string;
  timestamp: string;
}

export interface ModMonitorStatus {
  is_game_running: boolean;
  pid: number | null;
  loaded_mods: number;
  total_mods: number;
  error_count: number;
  warning_count: number;
  mod_load_events: ModLoadEvent[];
  error_events: ModErrorEvent[];
  warning_events: ModWarningEvent[];
  health_score: number;
}

export async function startGameMonitor(): Promise<boolean> {
  return invoke<boolean>('start_game_monitor');
}

export async function stopGameMonitor(): Promise<boolean> {
  return invoke<boolean>('stop_game_monitor');
}

export async function getMonitorStatus(totalMods: number): Promise<ModMonitorStatus> {
  return invoke<ModMonitorStatus>('get_monitor_status', { totalMods });
}

export async function listenToMonitorUpdates(callback: (data: ModMonitorStatus) => void): Promise<() => void> {
  const unlisten = await listen('mod-monitor-update', (event) => {
    callback(event.payload as ModMonitorStatus);
  });
  return unlisten;
}

export interface SecurityCheck {
  check_name: string;
  passed: boolean;
  severity: string;
  description: string;
}

export interface ModSecurityReport {
  mod_name: string;
  unique_id: string;
  security_score: number;
  risk_level: string;
  checks: SecurityCheck[];
  recommendations: string[];
}

export interface BatchSecurityReport {
  reports: ModSecurityReport[];
  average_score: number;
  high_risk_count: number;
  medium_risk_count: number;
  low_risk_count: number;
}

export async function checkModSecurity(modPath: string): Promise<ModSecurityReport> {
  return invoke<ModSecurityReport>('check_mod_security', { modPath });
}

export async function batchCheckModSecurity(mods: { folder_path: string }[]): Promise<BatchSecurityReport> {
  return invoke<BatchSecurityReport>('batch_check_mod_security', { mods });
}
