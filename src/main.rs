mod renderer_context;

use renderer_context::{RendererContext, Resolution, RenderPipelineDesc};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};


pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut ctx = RendererContext::new(
        &window, 
        Resolution {
            width: 640,
            height: 480
        }
    ).await;

    {
        let shader_src = include_str!("shaders/render.wgsl");
        let shader = ctx.new_shader(shader_src);
    
        let render_pipeline = ctx.new_render_pipeline(
            &RenderPipelineDesc {
                shader
            }
        );
    
        ctx.set_render_pipeline(render_pipeline);
    }
    
    {
        let shader_src = include_str!("shaders/compute.wgsl");
        let shader = ctx.new_shader(shader_src);
    
        let compute_pipeline = ctx.new_compute_pipeline(
            &RenderPipelineDesc {
                shader
            }
        );
    
        ctx.set_compute_pipeline(compute_pipeline);
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            ctx.commit_frame();
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Resized(physical_size) => {
                ctx.resize(Resolution { width: physical_size.width, height: physical_size.height });
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                ctx.resize(Resolution { width: new_inner_size.width, height: new_inner_size.height });
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });
}

fn main() {
    pollster::block_on(run());
}