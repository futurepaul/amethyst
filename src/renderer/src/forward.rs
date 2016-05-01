use gfx;
use gfx::traits::FactoryExt;
use std::collections::HashMap;
use mopa::Any;
use {Method, ScreenOutput};

pub use VertexPosNormal;

pub static VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    uniform mat4 u_Proj;
    uniform mat4 u_View;
    uniform mat4 u_Model;

    in vec3 a_Pos;
    in vec3 a_Normal;

    void main() {
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
";

pub static FLAT_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform vec4 u_Ka;

    out vec4 o_Ka;

    void main() {
        o_Ka = u_Ka;
    }
";

pub static WIREFRAME_GEOMETRY_SRC: &'static [u8] = b"
    #version 150 core

    layout(triangles) in;
    layout(line_strip, max_vertices=4) out;

    void main() {
        gl_Position = gl_in[0].gl_Position;
        EmitVertex();
        gl_Position = gl_in[1].gl_Position;
        EmitVertex();
        gl_Position = gl_in[2].gl_Position;
        EmitVertex();
        gl_Position = gl_in[0].gl_Position;
        EmitVertex();
        EndPrimitive();
    }
";

pub type GFormat = [f32; 4];

pub struct Clear;

impl<R, C> Method<::Clear, ScreenOutput<R>, R, C> for Clear
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          <R as gfx::Resources>::RenderTargetView: Any,
          <R as gfx::Resources>::Texture: Any,
          <R as gfx::Resources>::DepthStencilView: Any,
          R: 'static
{
    fn apply(&self, arg: &::Clear, target: &ScreenOutput<R>, _: &HashMap<String, ::Scene<R>>, encoder: &mut gfx::Encoder<R, C>) {
        encoder.clear(&target.output, arg.color);
        encoder.clear_depth(&target.output_depth, 1.0);
    }
}

gfx_pipeline!( flat {
    vbuf: gfx::VertexBuffer<VertexPosNormal> = (),
    ka: gfx::Global<[f32; 4]> = "u_Ka",
    model: gfx::Global<[[f32; 4]; 4]> = "u_Model",
    view: gfx::Global<[[f32; 4]; 4]> = "u_View",
    proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
    out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Ka",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
});

pub struct FlatShading<R: gfx::Resources>(gfx::pso::PipelineState<R, flat::Meta>);

impl<R: gfx::Resources> FlatShading<R> {
    pub fn new<F>(factory: &mut F) -> FlatShading<R>
        where R: gfx::Resources,
              F: gfx::Factory<R>
    {
        FlatShading(factory.create_pipeline_simple(
            VERTEX_SRC,
            FLAT_FRAGMENT_SRC,
            gfx::state::CullFace::Back,
            flat::new()
        ).unwrap())
    }
}

impl<R, C> Method<::FlatShading, ScreenOutput<R>, R, C> for FlatShading<R>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          <R as gfx::Resources>::RenderTargetView: Any,
          <R as gfx::Resources>::Texture: Any,
          <R as gfx::Resources>::DepthStencilView: Any,
          R: 'static
{
    fn apply(&self, arg: &::FlatShading, target: &ScreenOutput<R>, scenes: &HashMap<String, ::Scene<R>>, encoder: &mut gfx::Encoder<R, C>) {
        let scene = &scenes[&arg.scene];

        // every entity gets drawn
        for e in &scene.fragments {
            encoder.draw(
                &e.slice,
                &self.0,
                &flat::Data{
                    vbuf: e.buffer.clone(),
                    ka: e.ka,
                    model: e.transform,
                    view: arg.camera.view,
                    proj: arg.camera.projection,
                    out_ka: target.output.clone(),
                    out_depth: target.output_depth.clone()
                }
            );
        }
    }
}

gfx_pipeline!( wireframe {
    vbuf: gfx::VertexBuffer<VertexPosNormal> = (),
    ka: gfx::Global<[f32; 4]> = "u_Ka",
    model: gfx::Global<[[f32; 4]; 4]> = "u_Model",
    view: gfx::Global<[[f32; 4]; 4]> = "u_View",
    proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
    out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Ka",
});

pub struct Wireframe<R: gfx::Resources>(gfx::pso::PipelineState<R, wireframe::Meta>);

impl<R: gfx::Resources> Wireframe<R> {
    pub fn new<F>(factory: &mut F) -> Wireframe<R>
        where F: gfx::Factory<R>
    {
        let vs = factory.create_shader_vertex(VERTEX_SRC).unwrap();
        let gs = factory.create_shader_geometry(WIREFRAME_GEOMETRY_SRC).unwrap();
        let fs = factory.create_shader_pixel(FLAT_FRAGMENT_SRC).unwrap();

        Wireframe(factory.create_pipeline_state(
            &gfx::ShaderSet::Geometry(vs, gs, fs),
            gfx::Primitive::TriangleList,
            gfx::state::Rasterizer::new_fill(gfx::state::CullFace::Nothing),
            wireframe::new()
        ).unwrap())
    }
}



impl<R, C> Method<::Wireframe, ScreenOutput<R>, R, C> for Wireframe<R>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          <R as gfx::Resources>::RenderTargetView: Any,
          <R as gfx::Resources>::Texture: Any,
          <R as gfx::Resources>::DepthStencilView: Any,
          R: 'static
{
    fn apply(&self, arg: &::Wireframe, target: &ScreenOutput<R>, scenes: &HashMap<String, ::Scene<R>>, encoder: &mut gfx::Encoder<R, C>) {
        let scene = &scenes[&arg.scene];

        // every entity gets drawn
        for e in &scene.fragments {
            encoder.draw(
                &e.slice,
                &self.0,
                &wireframe::Data{
                    vbuf: e.buffer.clone(),
                    ka: e.ka,
                    model: e.transform,
                    view: arg.camera.view,
                    proj: arg.camera.projection,
                    out_ka: target.output.clone()
                }
            );
        }
    }
}
