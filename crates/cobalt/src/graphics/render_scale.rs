use unity::{engine::rendering::universal::UniversalRenderPipelineAsset, prelude::*};

use crate::sequences::mainmenu::cobaltmenu::sequences::settings::{render_scale::get_render_scale, render_scale_toggle};

// CustomRP.Settings$$SetRenderScale	71021a8db0	void CustomRP.Settings$$SetRenderScale(float scale, MethodInfo * method)	340
#[skyline::hook(offset = 0x21a8db0)]
pub fn set_render_scale(mut scale: f32, method_info: OptionalMethod) {
    if *render_scale_toggle::TOGGLE.read().unwrap() {
        scale = get_render_scale().unwrap_or(scale);
        println!("SetRenderScale: {}", scale);
    }
    
    call_original!(scale, method_info);
}

// UnityEngine.Rendering.Universal.UniversalRenderPipelineAsset$$set_renderScale	7102cafa50	void UnityEngine.Rendering.Universal.UniversalRenderPipelineAsset$$set_renderScale(UnityEngine_Rendering_Universal_UniversalRenderPipelineAsset_o * __this, float value, MethodInfo * method)	156
#[skyline::hook(offset = 0x2cafa50)]
pub fn set_render_scale_internal(mut asset: UniversalRenderPipelineAsset, value: f32, _method_info: OptionalMethod) {
    // ensure floor of scale is 0.1
    let scale = value.max(0.1);
    asset.render_scale = scale;
}
