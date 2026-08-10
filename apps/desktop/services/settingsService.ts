import { desktopApi, SettingsPayload, ValidationResult } from "./desktopApi";

export type SettingsLoadResult = SettingsPayload;

export async function loadSettings(): Promise<SettingsLoadResult> {
  return desktopApi.readSettingsJson();
}

export async function saveSettings(jsonText: string): Promise<SettingsLoadResult> {
  return desktopApi.writeSettingsJson(jsonText);
}

export async function validateSettings(jsonText: string): Promise<ValidationResult> {
  return desktopApi.validateSettingsJson(jsonText);
}

export async function resetSettings(): Promise<SettingsLoadResult> {
  return desktopApi.resetSettingsJson();
}

export async function openSettingsJsonInVSCode() {
  return desktopApi.openSettingsJsonInVSCode();
}
