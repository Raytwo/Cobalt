use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

use engage::unity_engine::{
    assetbundlerequest::{AssetBundleRequest, IAssetBundleRequestMethods},
    textasset::{ITextAssetMethods, TextAsset},
};
use unity::prelude::*;

use crate::api::events::{publish_system_event, SystemEvent};

pub mod loader;
pub mod synthesize;

static CATALOG_PROCESSED: AtomicBool = AtomicBool::new(false);

#[skyline::hook(offset = 0x2dc0c50)]
pub fn catalog_load_complete_hook(this: IlInstance, op: AssetBundleRequest, method_info: OptionalMethod) {
    if CATALOG_PROCESSED.load(Relaxed) {
        call_original!(this, op, method_info);
        publish_system_event(SystemEvent::CatalogLoaded);
        return;
    }

    let total_start = std::time::Instant::now();
    let mut handled = false;
    let mut game_ms = 0u128;

    if loader::synthesis_needed() {
        let text_start = std::time::Instant::now();
        if let Some(base_json) = base_catalog_text(op) {
            println!("[catalog] base catalog text: {}ms", text_start.elapsed().as_millis());
            if loader::synthesize_all(&base_json) {
                let game_start = std::time::Instant::now();
                call_original!(this, op, method_info);
                game_ms = game_start.elapsed().as_millis();
                loader::load_and_register();
                handled = true;
            } else {
                call_original!(this, op, method_info);
            }
        } else {
            call_original!(this, op, method_info);
        }
    } else {
        let game_start = std::time::Instant::now();
        call_original!(this, op, method_info);
        game_ms = game_start.elapsed().as_millis();
        loader::load_and_register();
        handled = true;
    }

    if handled {
        CATALOG_PROCESSED.store(true, Relaxed);
        let total_ms = total_start.elapsed().as_millis();
        println!("[catalog] game base-catalog load (call_original): {}ms", game_ms);
        println!("[catalog] TOTAL Cobalt overhead: {}ms", total_ms.saturating_sub(game_ms));
    }

    publish_system_event(SystemEvent::CatalogLoaded);
}

fn base_catalog_text(op: AssetBundleRequest) -> Option<String> {
    if op.is_null() {
        return None;
    }
    let asset = op.get_asset();
    if asset.is_null() {
        return None;
    }
    let text = TextAsset::from_il_instance(asset.as_instance()).get_text();
    if text.is_null() {
        return None;
    }
    Some(text.to_rust_string())
}
