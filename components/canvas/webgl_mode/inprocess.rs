use ::gl_context::GLContextFactory;
use ::webgl_thread::{WebGLExternalImageApi, WebGLExternalImageHandler, WebGLThreadObserver, WebGLThread};
use canvas_traits::webgl::{webgl_channel, WebGLChan, WebGLContextId};
use canvas_traits::webgl::{WebGLMsg, WebGLPipeline, WebGLReceiver, WebGLSender, WebVRCommand, WebVRRenderHandler};
use euclid::Size2D;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use webrender;
use webrender_traits;

type WebGLThreadId = u32;
type WebGLThreadMap = Arc<RefCell<HashMap<WebGLContextId, WebGLThreadId>>>;
type WebGLExternalImageMap = Arc<RefCell<HashMap<WebGLThreadId, WebGLExternalImageChannels>>>;

pub struct WebGLThreads {
    gl_factory: Arc<GLContextFactory>,
    webrender_api_sender: webrender_traits::RenderApiSender,
    webvr_compositor: Option<Box<WebVRRenderHandler>>,
    next_thread_id: WebGLThreadId,
    thread_map: WebGLThreadMap,
    external_image_map: WebGLExternalImageMap,
}

#[allow(unsafe_code)]
unsafe impl Send for WebGLThreads {}

impl WebGLThreads {
    pub fn new(gl_factory: GLContextFactory,
               webrender_api_sender: webrender_traits::RenderApiSender,
               webvr_compositor: Option<Box<WebVRRenderHandler>>) 
               -> (WebGLThreads, Box<webrender::ExternalImageHandler>) {
        let threads = WebGLThreads {
            gl_factory: Arc::new(gl_factory),
            webrender_api_sender: webrender_api_sender,
            webvr_compositor: webvr_compositor,
            next_thread_id: 0,
            thread_map: Arc::new(RefCell::new(HashMap::new())),
            external_image_map: Arc::new(RefCell::new(HashMap::new())),
        };

        let external = WebGLExternalImageHandler::new(WebGLExternalImages {
            external_image_map: threads.external_image_map.clone(),
            thread_map: threads.thread_map.clone(),
        });

        (threads, Box::new(external))
    }

    pub fn pipeline(&mut self) -> WebGLPipeline {
        let thread_id = self.next_thread_id;
        self.next_thread_id += 1;

        let observer = WebGLThreadObserverData {
            thread_id: thread_id,
            thread_map: self.thread_map.clone(),
        };

        let thiz = self as *mut _;
        let channel = WebGLThread::start(self.gl_factory.clone(), 
                                         self.webrender_api_sender.clone(),
                                         self.webvr_compositor.as_mut().map(|_| WebVRRenderWrapper(thiz)),
                                         observer);
        let external = WebGLExternalImageChannels {
            webgl_channel: channel.clone(),
            lock_channel: webgl_channel().unwrap()
        };

        self.external_image_map.borrow_mut().insert(thread_id, external);
        
        WebGLPipeline(WebGLChan(channel))
    }

    pub fn exit(&self) -> Result<(), String> {
        let mut n = 0;
        for (_, channels) in self.external_image_map.borrow().iter() {
            if channels.webgl_channel.send(WebGLMsg::Exit).is_err() {
                n += 1;
            }
        }

        if n > 0 {
            Err(format!("Failed to send exit message to {} WebGLThreads", n))
        } else {
            Ok(())
        }
    }
}

// WebRender External Image API
struct WebGLExternalImageChannels {
    webgl_channel: WebGLSender<WebGLMsg>,
    // Used to avoid creating a new channel on each received WebRender request.
    lock_channel: (WebGLSender<(u32, Size2D<i32>)>, WebGLReceiver<(u32, Size2D<i32>)>),
}

struct WebGLExternalImages {
    external_image_map: WebGLExternalImageMap,
    thread_map: WebGLThreadMap,
}

impl WebGLExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, ctx_id: WebGLContextId) -> (u32, Size2D<i32>) {
        let thread_id = self.thread_map.borrow()[&ctx_id];
        let external_image_map = self.external_image_map.borrow();
        let ctx_channels = external_image_map.get(&thread_id).unwrap();
        ctx_channels.webgl_channel.send(WebGLMsg::Lock(ctx_id, ctx_channels.lock_channel.0.clone())).unwrap();
        ctx_channels.lock_channel.1.recv().unwrap()
    }

    fn unlock(&mut self, ctx_id: WebGLContextId) {
        if let Some(thread_id) = self.thread_map.borrow().get(&ctx_id) {
            if let Some(ctx_channels) = self.external_image_map.borrow().get(&thread_id) {
                ctx_channels.webgl_channel.send(WebGLMsg::Unlock(ctx_id)).unwrap();
            }
        }
    }
}

// Observer 
struct WebGLThreadObserverData {
    thread_id: WebGLThreadId,
    thread_map: WebGLThreadMap,
}

#[allow(unsafe_code)]
unsafe impl Send for WebGLThreadObserverData {}

impl WebGLThreadObserver for WebGLThreadObserverData {
    fn on_context_create(&mut self, ctx_id: WebGLContextId, _texture_id: u32, _size: Size2D<i32>) {
        self.thread_map.borrow_mut().insert(ctx_id, self.thread_id);
    }

    fn on_context_resize(&mut self, _ctx_id: WebGLContextId, _texture_id: u32, _size: Size2D<i32>) {
        // No op
    }

    fn on_context_delete(&mut self, ctx_id: WebGLContextId) {
        self.thread_map.borrow_mut().remove(&ctx_id);
    }
}

// Wrapper for WebVRRenderHandler
struct WebVRRenderWrapper(*mut WebGLThreads);

#[allow(unsafe_code)]
unsafe impl Send for WebVRRenderWrapper {}

impl WebVRRenderHandler for WebVRRenderWrapper {
    #[allow(unsafe_code)]
    fn handle(&mut self, command: WebVRCommand, texture: Option<(u32, Size2D<i32>)>) {
        unsafe {
            (*self.0).webvr_compositor.as_mut().unwrap().handle(command, texture);
        }
    }
}
