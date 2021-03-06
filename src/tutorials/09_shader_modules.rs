
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::share,
    utility::debug::*,
    utility::constants::*,
};

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::InstanceV1_0;
use ash::version::DeviceV1_0;

use std::path::Path;
use std::ptr;
use std::ffi::CString;

// Constants
const WINDOW_TITLE: &'static str = "09.Shader Modules";

struct VulkanApp {
    // winit stuff
    events_loop          : EventsLoop,
    _window              : winit::Window,

    // vulkan stuff
    _entry               : ash::Entry,
    instance             : ash::Instance,
    surface_loader       : ash::extensions::Surface,
    surface              : vk::SurfaceKHR,
    debug_report_loader  : ash::extensions::DebugReport,
    debug_callback       : vk::DebugReportCallbackEXT,

    _physical_device     : vk::PhysicalDevice,
    device               : ash::Device,

    _graphics_queue      : vk::Queue,
    _present_queue       : vk::Queue,

    swapchain_loader     : ash::extensions::Swapchain,
    swapchain            : vk::SwapchainKHR,
    _swapchain_images    : Vec<vk::Image>,
    _swapchain_format    : vk::Format,
    _swapchain_extent    : vk::Extent2D,
    swapchain_imageviews : Vec<vk::ImageView>,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = utility::window::init_window(&events_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = ash::Entry::new().unwrap();
        let instance = share::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = share::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_report_loader, debug_callback) = setup_debug_callback( VALIDATION.is_enable, &entry, &instance);
        let physical_device = share::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let (device, family_indices) = share::create_logical_device(&instance, physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(family_indices.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(family_indices.present_family as u32, 0) };
        let swapchain_stuff = share::create_swapchain(&instance, &device, physical_device, &window, &surface_stuff, &family_indices);
        let swapchain_imageviews = share::v1::create_image_views(&device, swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let _pipeline = VulkanApp::create_graphics_pipeline(&device);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            // winit stuff
            events_loop,
            _window: window,

            // vulkan stuff
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_report_loader,
            debug_callback,

            _physical_device: physical_device,
            device,

            _graphics_queue: graphics_queue,
            _present_queue : present_queue,

            swapchain_loader : swapchain_stuff.swapchain_loader,
            swapchain        : swapchain_stuff.swapchain,
            _swapchain_format: swapchain_stuff.swapchain_format,
            _swapchain_images: swapchain_stuff.swapchain_images,
            _swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,
        }
    }

    fn create_graphics_pipeline(device: &ash::Device) {

        let vert_shader_code = VulkanApp::read_shader_code(Path::new("shaders/spv/09-shader-base.vert.spv"));
        let frag_shader_code = VulkanApp::read_shader_code(Path::new("shaders/spv/09-shader-base.frag.spv"));

        let vert_shader_module = VulkanApp::create_shader_module(device, vert_shader_code);
        let frag_shader_module = VulkanApp::create_shader_module(device, frag_shader_code);

        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let _shader_stages = [
            vk::PipelineShaderStageCreateInfo { // Vertex Shader
                s_type                : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next                : ptr::null(),
                flags                 : vk::PipelineShaderStageCreateFlags::empty(),
                module                : vert_shader_module,
                p_name                : main_function_name.as_ptr(),
                p_specialization_info : ptr::null(),
                stage                 : vk::ShaderStageFlags::VERTEX,
            },
            vk::PipelineShaderStageCreateInfo { // Fragment Shader
                s_type                : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next                : ptr::null(),
                flags                 : vk::PipelineShaderStageCreateFlags::empty(),
                module                : frag_shader_module,
                p_name                : main_function_name.as_ptr(),
                p_specialization_info : ptr::null(),
                stage                 : vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }
    }

    fn create_shader_module(device: &ash::Device, code: Vec<u8>) -> vk::ShaderModule {

        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type    : vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next    : ptr::null(),
            flags     : vk::ShaderModuleCreateFlags::empty(),
            code_size : code.len(),
            p_code    : code.as_ptr() as *const u32,
        };

        unsafe {
            device.create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create Shader Module!")
        }
    }

    fn read_shader_code(shader_path: &Path) -> Vec<u8> {
        use std::fs::File;
        use std::io::Read;

        let spv_file = File::open(shader_path)
            .expect(&format!("Failed to find spv file at {:?}", shader_path));
        let bytes_code: Vec<u8> = spv_file.bytes()
            .filter_map(|byte| byte.ok()).collect();

        bytes_code
    }
}

impl Drop for VulkanApp {

    fn drop(&mut self) {

        unsafe {

            for &imageview in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(imageview, None);
            }

            self.swapchain_loader.destroy_swapchain_khr(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface_khr(self.surface, None);

            if VALIDATION.is_enable {
                self.debug_report_loader.destroy_debug_report_callback_ext(self.debug_callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}




// Fix content -------------------------------------------------------------------------------
impl VulkanApp {

    pub fn main_loop(&mut self) {

        self.events_loop.run_forever(|event| {

            match event {
                // handling keyboard event
                | Event::WindowEvent { event, .. } => match event {
                    | WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            ControlFlow::Break
                        } else {
                            ControlFlow::Continue
                        }
                    }
                    | WindowEvent::CloseRequested => ControlFlow::Break,
                    | _ => ControlFlow::Continue,
                },
                | _ => ControlFlow::Continue,
            }
        });
    }
}

fn main() {

    let mut vulkan_app = VulkanApp::new();
    vulkan_app.main_loop();
}
// -------------------------------------------------------------------------------------------
