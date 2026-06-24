use unity::OptionalMethod;

use engage::GameVariableManager;
use crate::config::{combatpopup::DISABLE_COMBAT_POPUPS_KEY, combatui::DISABLE_COMBAT_UI_KEY};

// Combat.HUDPopupGroup$$DamagePopup	7102979280	void Combat.HUDPopupGroup$$DamagePopup(Combat_Phase_o * phase, MethodInfo * method)	584
#[unity::hook("Combat", "HUDPopupGroup", "DamagePopup")]
pub fn damage_popup(combat_phase: *const u8, method_info: OptionalMethod) {
    if GameVariableManager::get_bool(DISABLE_COMBAT_POPUPS_KEY) {
        return;
    }
    call_original!(combat_phase, method_info);
}

// Combat.CombatWorld$$FadeInHUD	71029368f0	void Combat.CombatWorld$$FadeInHUD(Combat_CombatWorld_o * __this, MethodInfo * method)	320
#[unity::hook("Combat", "CombatWorld", "FadeInHUD")]
pub fn fade_in_hud(this: *const u8, method_info: OptionalMethod) {
    if GameVariableManager::get_bool(DISABLE_COMBAT_UI_KEY) {
        return;
    }
    call_original!(this, method_info);
}
