// harper-desktop/src/lib/settings/accessibilityConsent.js
import { invoke } from '@tauri-apps/api/tauri';

/**
 * Ensure user has consented to accessibility usage for the given bundleId.
 * Returns true when consent is present (or granted through the prompt).
 * Returns false otherwise.
 */
export async function ensureAccessibilityConsent(bundleId) {
  if (!bundleId) return false;

  try {
    const already = await invoke('has_user_consented', { bundleId });
    if (already) {
      return true;
    }

    // Replace window.confirm with your modal dialog for a better UX.
    const ok = window.confirm(
      `Harper needs permission to read text from ${bundleId} to provide accessibility highlights for that app. Do you allow this?`
    );

    if (!ok) {
      return false;
    }

    // Persist user's approval
    await invoke('set_user_consent_cmd', { bundleId, allow: true });
    return true;
  } catch (e) {
    console.error('Consent API error', e);
    return false;
  }
}
