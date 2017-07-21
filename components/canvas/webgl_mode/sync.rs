use ::gl_context::GLContextFactory;
use ::webgl_thread::{WebGLExternalImageApi, WebGLExternalImageHandler, WebGLThreadObserver, WebGLThread};
use canvas_traits::webgl::{WebGLChan, WebGLContextId};
use canvas_traits::webgl::{WebGLMsg, WebGLPipeline, WebGLSyncCall, WebGLSyncPipeline, WebVRCommand, WebVRRenderHandler};
use euclid::Size2D;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;
use webrender;
use webrender_traits;

type WebGLExternalImageMap = Arc<RefCell<HashMap<WebGLContextId, (u32, Size2D<i32>)>>>;

pub struct WebGLThreads {
    gl_factory: Arc<GLContextFactory>,
    webrender_api_sender: webrender_traits::RenderApiSender,
    webvr_compositor: Option<Box<WebVRRenderHandler>>,
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
            external_image_map: Arc::new(RefCell::new(HashMap::new())),
        };

        let external = WebGLExternalImageHandler::new(WebGLExternalImages(threads.external_image_map.clone()));

        (threads, Box::new(external))
    }

    pub fn pipeline(&mut self) -> WebGLPipeline {
        WebGLPipeline(Box::new(WebGLThreadsHandle(self as * mut _)))
    }

    pub fn exit(&self) -> Result<(), String> {
        // No op
        Ok(())
    }

    pub fn channel(&mut self) -> WebGLChan {
        let observer = WebGLThreadObserverData(self.external_image_map.clone());
        let thiz = self as *mut _;
        let renderer = WebGLThread::new(self.gl_factory.clone(), 
                                        self.webrender_api_sender.clone(),
                                        self.webvr_compositor.as_mut().map(|_| WebVRRenderWrapper(thiz)),
                                        observer);

        WebGLChan(Rc::new(RefCell::new(renderer)))
    }
}

// Trait used to create the WebGLMsg channel
#[derive(Clone)]
struct WebGLThreadsHandle(*mut WebGLThreads);

#[allow(unsafe_code)]
unsafe impl Send for WebGLThreadsHandle {}

impl WebGLSyncPipeline for WebGLThreadsHandle {
    #[allow(unsafe_code)]
    fn channel(&self) -> WebGLChan {
         unsafe { (*self.0).channel() }
    }
}

// Trait used to call WebGL methods
impl<VR: WebVRRenderHandler + 'static, OB: WebGLThreadObserver> WebGLSyncCall for WebGLThread<VR, OB> {
    fn call(&mut self, msg: WebGLMsg, c: &WebGLChan) {
        self.handle_msg(msg, c);
    }
}

struct WebGLExternalImages(WebGLExternalImageMap);

impl WebGLExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, ctx_id: WebGLContextId) -> (u32, Size2D<i32>) {
        *self.0.borrow().get(&ctx_id).unwrap()
    }

    fn unlock(&mut self, ctx_id: WebGLContextId) {
        // No op
    }
}

// Observer 
struct WebGLThreadObserverData(WebGLExternalImageMap);

#[allow(unsafe_code)]
unsafe impl Send for WebGLThreadObserverData {}

impl WebGLThreadObserver for WebGLThreadObserverData {
    fn on_context_create(&mut self, ctx_id: WebGLContextId, texture_id: u32, size: Size2D<i32>) {
        self.0.borrow_mut().insert(ctx_id, (texture_id, size));
    }

    fn on_context_resize(&mut self, ctx_id: WebGLContextId, texture_id: u32, size: Size2D<i32>) {
        self.0.borrow_mut().insert(ctx_id, (texture_id, size));
    }

    fn on_context_delete(&mut self, ctx_id: WebGLContextId) {
        self.0.borrow_mut().remove(&ctx_id);
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
