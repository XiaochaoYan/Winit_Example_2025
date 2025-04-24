mod Hardware_Wrapper;

use std::sync::Arc;
use std::thread::sleep;
use wgpu::Adapter;
use wgpu::hal::DynInstance;
use wgpu::SurfaceTarget::Window as WGPU_Window;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event::{ButtonId, DeviceEvent, DeviceId, ElementState, Event, MouseButton, WindowEvent};
use winit::event::DeviceEvent::Button;
use winit::event::Event::UserEvent;
use winit::event_loop;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::keyboard::NamedKey::Print;
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{Window, WindowId, WindowAttributes};
use winit::platform::windows::Color;
use Hardware_Wrapper::HardwareWrapper;


const bool_tranparent: bool = false;

struct App<'a> {
    window: Option<Arc<Window>>,
    counter:u32,
    initialized_bool:bool,
    event_proxy:Option<EventLoopProxy<MyUserEvent>>,
    event_loop:Option<EventLoop<()>>,
    hardware_wrapper: Option<HardwareWrapper <'a>>,
}

#[derive(Debug)]
enum MyUserEvent {
    Mouseclick(MouseButton),
    WindowResize(PhysicalSize<u32>),
}

impl App<'_> {
    fn default() -> Self {
        App
        {
            counter: 0,
            window: None,
            initialized_bool: false,
            event_proxy: None,
            event_loop: None,
            hardware_wrapper: None,
        }
    }
}

impl Default for MyUserEvent {
    fn default() -> Self{
        MyUserEvent::Mouseclick(MouseButton::Left)
    }
}

impl MyUserEvent {
    fn new(mouse_button: MouseButton) ->Self
    {
        return MyUserEvent::Mouseclick(mouse_button);
    }
}

impl ApplicationHandler<MyUserEvent> for App<'_>{

    fn user_event(&mut self, event_loop: &ActiveEventLoop, user_event: MyUserEvent) {
        // Handle user event.

        match user_event {
            MyUserEvent::Mouseclick(MouseButton::Left) => {
                println!("Button Left is pressed, This is a user event response.");
            },

            MyUserEvent::WindowResize(PhysicalSize { width, height }) =>
                {
                    println!("Window Resized! New Size {} x {} .",width, height);



                },

            _ => {
                println!("Arbitrary user event detected. This is a user event response.");
            }
        }

    }


    fn resumed(&mut self, event_loop: &ActiveEventLoop) {



        let primary_monitor = event_loop.primary_monitor();

        let mut window_attributes = Window::default_attributes()
            .with_title("Winit window")
            .with_transparent(bool_tranparent).
            with_border_color(Some(Color::from_rgb(128, 255, 255)))
            .with_inner_size(LogicalSize::new(primary_monitor.as_ref().unwrap().size().width/2, primary_monitor.as_ref().unwrap().size().height/2));

        self.window = Some(Arc::new(event_loop.create_window(window_attributes).expect("Window failed to be created")));

        let hardware_wrapper = HardwareWrapper::new_winit(self.window.as_ref().unwrap().clone());
        self.hardware_wrapper = Some(pollster::block_on(async { hardware_wrapper.await}));

        println!("{:}", self.hardware_wrapper.as_ref().unwrap().adapter.get_info().name);




        println!("Window created on resume successfully.");
    }//TODO:need to implement windows
    //This seems to be there windows is created


    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Handle window event.
        match event {

            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },

            WindowEvent::Resized(size) =>
                {
                    self.event_proxy.as_ref().unwrap().send_event(MyUserEvent::WindowResize(size)).expect("Failed to send event");
                    let new_size = (size.width, size.height);
                    self.hardware_wrapper.as_mut().unwrap().resize_surface((size.width,size.height));
                    self.hardware_wrapper.as_mut().unwrap().render();
                    self.window.as_mut().unwrap().request_redraw();

                }

            WindowEvent::RedrawRequested => {
                if (!self.initialized_bool){
                    //TODO:Do initialization process here
                    self.initialized_bool = true;
                }else
                {
                    //TODO:Rendering goes here
                }
                self.window.as_ref().unwrap().request_redraw();
            }//MATCH case: redraw requested

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        //
        match event {
            DeviceEvent::Button { button, state } => {
                if state.is_pressed() {
                    println!("Button {:?} pressed", button);
                }

                if button.eq(&0)
                {
                    self.event_proxy.as_ref().unwrap().send_event(MyUserEvent::Mouseclick(MouseButton::Left)).expect("Failed to send event");
                }
                else
                {
                    self.event_proxy.as_ref().unwrap().send_event(MyUserEvent::Mouseclick(MouseButton::Other(99))).expect("Failed to send event");
                }

            }

            DeviceEvent::Key(event) => {
                if event.state == ElementState::Released {
                    println!("{:?} is press and released.", event.physical_key);

                    if (event.physical_key == PhysicalKey::Code(KeyCode::Escape))
                    {
                        event_loop.exit();
                    }

                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
        self.counter += 1;
    }
}


impl App<'_>{

    async fn request_device(&mut self)
    {
        if (self.window.is_none())
        {
            //TODO:set up windows
            //we assume Arc is already initialized
        }

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), ..Default::default()
        };
        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface = instance.create_surface(self.window.as_ref().unwrap()).
            expect("Couldn't create surface");

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
    }

}


fn main() {
    //let event_loop = EventLoop::new().unwrap();

    //let event_loop = EventLoop::with_user_event().build().unwrap();
    let event_loop =  EventLoop::<MyUserEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    //event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    app.event_proxy = Some(proxy);


    let _ = event_loop.run_app(&mut app);
}
