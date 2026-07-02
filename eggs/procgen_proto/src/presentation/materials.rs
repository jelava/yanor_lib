use bevy::{
    mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
    pbr::{MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PixelArtMaterialExtension {
    #[texture(50, dimension = "2d_array")]
    #[sampler(51)]
    pub color_texture_array: Handle<Image>,
}

impl PixelArtMaterialExtension {
    pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute = MeshVertexAttribute::new(
        "TextureIndex",
        Mesh::FIRST_AVAILABLE_CUSTOM_ATTRIBUTE,
        VertexFormat::Uint32,
    );
}

impl MaterialExtension for PixelArtMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        "shaders/pixel_art_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/pixel_art_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Self::ATTRIBUTE_TEXTURE_INDEX.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
