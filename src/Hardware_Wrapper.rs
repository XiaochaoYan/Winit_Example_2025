use std::sync::Arc;
use winit::window::Window;
use wgpu;
use wgpu::{Color, Features, StoreOp, Surface};
use wgpu::SurfaceConfiguration;

const default_clearing_color : wgpu::Color =  wgpu::Color {
    r: 253.0 / 255.0,
    g: 56.0 / 255.0,
    b: 122.0 / 255.0,
    a: 1.0,
};

pub struct HardwareWrapper<'a>
{
    pub instance: wgpu::Instance,
    pub surface: Surface<'a>,
    pub device: Arc<wgpu::Device>,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub config: Option<SurfaceConfiguration>,
    pub size: (u32, u32),
    pub window: Arc<Window>,
}

impl <'a>  HardwareWrapper<'a>
{
    pub async fn new_winit(window: Arc<Window>) -> HardwareWrapper <'a>
    {

        let local_window = window.clone();
        let size = (local_window.inner_size().width, local_window.inner_size().height);

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), ..Default::default()
        };
        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface =  instance.create_surface(window.clone()).expect("Surface failed to create");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to request an appropriate adapter");

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
            ..Default::default()
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor).
            await.
            expect("Failed to request an appropriate device");

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f | f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device, &config);


        return  Self {
            instance,
            surface: surface,
            device:Arc::new(device),
            adapter,
            queue,
            config:Some(config),
            size,
            window: window.clone(),
        }

    }


    pub fn check_feature_support(&self,feature_to_check:Features) ->bool
    {
        return self.adapter.features().contains(feature_to_check);
    }

    pub fn set_config(&mut self, new_config: wgpu::SurfaceConfiguration) {
        self.config = Some(new_config);
    }

    pub fn set_surface(&mut self, new_surface: wgpu::Surface<'a>) {
        self.surface = new_surface;
    }

    pub fn resize_surface(&mut self, new_size: (u32, u32))
    {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.as_mut().unwrap().width = new_size.0 as u32;
            self.config.as_mut().unwrap().height = new_size.1 as u32;
            self.surface.configure(&self.device, &self.config.as_ref().unwrap());
        }
    }

    pub fn update_surface(&mut self) {
        self.surface = self.instance.create_surface(self.window.clone()).expect("Surface failed to create");
    }

    pub fn get_window(&mut self) -> & Window
    {
        return self.window.as_ref();
    }


    pub fn render(&mut self) {
        let output = self.surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture");

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Basic Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(default_clearing_color), // or whatever color you want
                        store: StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // No drawing yet — just clearing
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
    }

}
