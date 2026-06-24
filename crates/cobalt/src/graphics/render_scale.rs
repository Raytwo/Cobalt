use engage::unity_engine::rendering::universal::universalrenderpipelineasset::UniversalRenderPipelineAsset;
use unity::{ClassIdentity, OptionalMethod};

use crate::sequences::mainmenu::cobaltmenu::sequences::settings::{
    render_scale::CURRENT_RENDER_SCALE, render_scale_toggle,
};

// CustomRP.Settings$$SetRenderScale	71021a8db0	void CustomRP.Settings$$SetRenderScale(float scale, MethodInfo * method)	340
#[unity::hook("CustomRP", "Settings", "SetRenderScale")]
pub fn set_render_scale(mut scale: f32, method_info: OptionalMethod) {
    if *render_scale_toggle::TOGGLE.read().unwrap() {
        scale = *CURRENT_RENDER_SCALE.read().unwrap();
        println!("SetRenderScale: {}", scale);
    }

    call_original!(scale, method_info);
}

// UnityEngine.Rendering.Universal.UniversalRenderPipelineAsset$$set_renderScale	7102cafa50	void UnityEngine.Rendering.Universal.UniversalRenderPipelineAsset$$set_renderScale(UnityEngine_Rendering_Universal_UniversalRenderPipelineAsset_o * __this, float value, MethodInfo * method)	156
#[unity::hook("UnityEngine.Rendering.Universal", "UniversalRenderPipelineAsset", "set_renderScale")]
pub fn set_render_scale_internal(this: UniversalRenderPipelineAsset, value: f32, _method_info: OptionalMethod) {
    use std::sync::OnceLock;
    static OFFSET: OnceLock<usize> = OnceLock::new();
    let offset = *OFFSET.get_or_init(|| {
        let class = <UniversalRenderPipelineAsset as ClassIdentity>::class();
        unity::class_get_field_from_name(class.raw(), "m_RenderScale").offset as usize
    });
    unity::field_set_value_at_offset::<f32>(this, offset, value.max(0.1));
}
