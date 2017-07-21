use ::webgl::WebGLMsg;
use ipc_channel;
use serde::{Deserialize, Serialize};
use std::io;

pub type WebGLSender<T> = ipc_channel::ipc::IpcSender<T>;
pub type WebGLReceiver<T> = ipc_channel::ipc::IpcReceiver<T>;
pub type WebGLSendResult = Result<(), ipc_channel::Error>;

pub fn webgl_channel<T: Serialize + for<'de> Deserialize<'de>>() -> Result<(WebGLSender<T>, WebGLReceiver<T>), io::Error> {
    ipc_channel::ipc::channel()
}

#[derive(Clone, Deserialize, Serialize)]
pub struct WebGLPipeline(pub WebGLChan);

impl WebGLPipeline {
    pub fn channel(&self) -> WebGLChan {
        self.0.clone()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct WebGLChan(pub WebGLSender<WebGLMsg>);

impl WebGLChan {
    #[inline]
    pub fn send(&self, msg: WebGLMsg) -> WebGLSendResult {
        self.0.send(msg)
    }
}
