use engage::gamevariable::GameVariableManager;
use unity::prelude::*;

use crate::config::{combatpopup::DISABLE_COMBAT_POPUPS_KEY, combatui::DISABLE_COMBAT_UI_KEY};

// Combat.HUDPopupGroup$$DamagePopup	7102979280	void Combat.HUDPopupGroup$$DamagePopup(Combat_Phase_o * phase, MethodInfo * method)	584
#[skyline::hook(offset = 0x2979280)]
pub fn damage_popup(combat_phase: *const u8, method_info: OptionalMethod) {
    let popup_disabled = GameVariableManager::get_bool(DISABLE_COMBAT_POPUPS_KEY);
    if popup_disabled {
        return
    }
    call_original!(combat_phase, method_info);
}

// Combat.CombatWorld$$FadeInHUD	71029368f0	void Combat.CombatWorld$$FadeInHUD(Combat_CombatWorld_o * __this, MethodInfo * method)	320
#[skyline::hook(offset = 0x29368f0)]
pub fn fade_in_hud(this: *const u8, method_info: OptionalMethod) {
    let ui_disabled = GameVariableManager::get_bool(DISABLE_COMBAT_UI_KEY);
    if ui_disabled {
        return
    }
    call_original!(this, method_info);
}
