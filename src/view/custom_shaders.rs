use ahash::AHashMap;
use rootvg::pipeline::{CustomPipeline, CustomPipelineID};

/// A collection of pipelines for custom shaders.
pub struct CustomPipelines {
    pipeline_ids: AHashMap<&'static str, CustomPipelineID>,
}

impl CustomPipelines {
    pub(crate) fn new() -> Self {
        Self {
            pipeline_ids: AHashMap::default(),
        }
    }

    pub fn get_id<P: CustomPipeline>(
        &mut self,
        id: &'static str,
        create_pipeline: impl FnOnce() -> P,
        vg: &mut rootvg::CanvasCtx<'_>,
    ) -> CustomPipelineID {
        let entry = self.pipeline_ids.entry(id);
        *entry.or_insert_with(|| vg.insert_custom_pipeline((create_pipeline)()))
    }
}
