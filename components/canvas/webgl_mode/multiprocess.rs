use ::gl_context::GLContextFactory;
use ::webgl_thread::{WebGLExternalImageApi, WebGLExternalImageHandler, WebGLThreadObserver, WebGLThread};
use canvas_traits::webgl::{webgl_channel, WebGLChan, WebGLContextId};
use canvas_traits::webgl::{WebGLMsg, WebGLPipeline, WebGLReceiver, WebGLSender, WebGLSendResult, WebVRCommand, WebVRRenderHandler};
use euclid::Size2D;
use std::marker::PhantomData;
use std::sync::Arc;
use webrender;
use webrender_api;

pub struct WebGLThreads(WebGLSender<WebGLMsg>);

impl WebGLThreads {
    pub fn new(gl_factory: GLContextFactory,
               webrender_api_sender: webrender_api::RenderApiSender,
               webvr_compositor: Option<Box<WebVRRenderHandler>>)
               -> (WebGLThreads, Box<webrender::ExternalImageHandler>) {
        let channel = WebGLThread::start(Arc::new(gl_factory), 
                                         webrender_api_sender,
                                         webvr_compositor.map(|c| WebVRRenderWrapper(c)),
                                         PhantomData);
        let external = WebGLExternalImageHandler::new(WebGLExternalImages::new(channel.clone()));
        (WebGLThreads(channel), Box::new(external))
    }

    pub fn pipeline(&self) -> WebGLPipeline {
        WebGLPipeline(WebGLChan(self.0.clone()))
    }

    pub fn exit(&self) -> WebGLSendResult {
        self.0.send(WebGLMsg::Exit)
    }
}

struct WebGLExternalImages {
    webgl_channel: WebGLSender<WebGLMsg>,
    // Used to avoid creating a new channel on each received WebRender request.
    lock_channel: (WebGLSender<(u32, Size2D<i32>)>, WebGLReceiver<(u32, Size2D<i32>)>),
}

impl WebGLExternalImages {
    fn new(channel: WebGLSender<WebGLMsg>) -> Self {
        Self {
            webgl_channel: channel,
            lock_channel: webgl_channel().unwrap(),
        }
    }
}

impl WebGLExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, ctx_id: WebGLContextId) -> (u32, Size2D<i32>) {
        self.webgl_channel.send(WebGLMsg::Lock(ctx_id, self.lock_channel.0.clone())).unwrap();
        self.lock_channel.1.recv().unwrap()
    }

    fn unlock(&mut self, ctx_id: WebGLContextId) {
        self.webgl_channel.send(WebGLMsg::Unlock(ctx_id)).unwrap();
    }
}

// No need to use Observer in this implementation
impl WebGLThreadObserver for PhantomData<()> {
    fn on_context_create(&mut self, _ctx_id: WebGLContextId, _texture_id: u32, _size: Size2D<i32>) {

    }

    fn on_context_resize(&mut self, _ctx_id: WebGLContextId, _texture_id: u32, _size: Size2D<i32>) {

    }

    fn on_context_delete(&mut self, _ctx_id: WebGLContextId) {

    }
}


// Wrapper for WebVRRenderHandler
struct WebVRRenderWrapper(Box<WebVRRenderHandler>);

impl WebVRRenderHandler for WebVRRenderWrapper {
    fn handle(&mut self, command: WebVRCommand, texture: Option<(u32, Size2D<i32>)>) {
        self.0.handle(command, texture);
    }
}