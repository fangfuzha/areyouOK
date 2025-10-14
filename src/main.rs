use std::num::NonZeroU32;
use std::rc::Rc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct App {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl ApplicationHandler for App {
    /// 当应用程序恢复时调用此函数
    ///
    /// 在应用程序启动或从暂停状态恢复时会调用此方法。主要负责创建窗口、
    /// 初始化图形上下文和表面，并触发首次重绘。
    ///
    /// 参数:
    /// - event_loop: 活动事件循环引用，用于创建窗口
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // 窗口未创建时初始化窗口及相关资源
            let window_attributes = WindowAttributes::default().with_title("areyouOK");
            let window = Rc::new(event_loop.create_window(window_attributes).unwrap());
            let context = Context::new(window.clone()).expect("创建 softbuffer 上下文失败");
            let surface = Surface::new(&context, window.clone()).expect("创建 softbuffer 表面失败");
            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);
        }

        // 确保窗口存在后请求重绘
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
    /// 处理窗口事件
    ///
    /// 根据不同的窗口事件执行相应的操作，包括关闭窗口、调整大小和重绘等。
    ///
    /// 参数:
    /// - event_loop: 活动事件循环引用，可用于控制应用生命周期
    /// - _id: 窗口ID（当前未使用）
    /// - event: 发生的窗口事件类型
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Window closed");
                event_loop.exit();
            }
            WindowEvent::Resized(_size) => {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.draw_frame() {
                    eprintln!("绘制首帧失败: {err}");
                }
            }
            _ => {}
        }
    }
}

impl App {
    /// 绘制一帧图像到窗口表面
    ///
    /// 此函数负责创建一个渐变背景并将其呈现到窗口中。它会检查窗口和表面是否存在，
    /// 调整表面大小以匹配窗口尺寸，然后在缓冲区中生成一个从左上到右下的彩色渐变，
    /// 最后将缓冲区内容呈现到屏幕上。
    ///
    /// 返回值:
    /// - Ok(()) 表示成功绘制并呈现了帧
    /// - Err(softbuffer::SoftBufferError) 表示在绘制过程中发生了错误
    fn draw_frame(&mut self) -> Result<(), softbuffer::SoftBufferError> {
        let window = match self.window.as_ref() {
            Some(window) => window,
            None => return Ok(()),
        };
        let surface = match self.surface.as_mut() {
            Some(surface) => surface,
            None => return Ok(()),
        };

        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }

        let width = size.width.min(u32::from(u16::MAX));
        let height = size.height.min(u32::from(u16::MAX));
        surface.resize(
            NonZeroU32::new(width).expect("宽度不能为空"),
            NonZeroU32::new(height).expect("高度不能为空"),
        )?;

        let mut buffer = surface.buffer_mut()?;
        let width_usize = width as usize;
        let height_usize = height as usize;
        for y in 0..height_usize {
            for x in 0..width_usize {
                let idx = y * width_usize + x;
                let r = (x * 255 / width_usize) as u32;
                let g = (y * 255 / height_usize) as u32;
                let b = 128u32;
                buffer[idx] = 0xFF00_0000 | (r << 16) | (g << 8) | b; // 简单的渐变填充
            }
        }
        buffer.present()?;
        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        window: None,
        context: None,
        surface: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
