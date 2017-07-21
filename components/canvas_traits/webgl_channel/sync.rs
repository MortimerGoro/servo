use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use ::webgl::WebGLMsg;

#[macro_use]
macro_rules! unreachable_serializable {
    ($name:ident) => {
        impl<T> Serialize for $name<T> {
            fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                unreachable!();
            }
        }

        impl<'a, T> Deserialize<'a> for $name<T> {
            fn deserialize<D>(_: D) -> Result<$name<T>, D::Error>
                            where D: Deserializer<'a> {
                unreachable!();
            }
        }
    };
}

#[derive(Clone)]
pub struct WebGLSender<T>(mpsc::Sender<T>);
pub struct WebGLReceiver<T>(mpsc::Receiver<T>);
pub type WebGLSendResult = Result<(), mpsc::SendError<WebGLMsg>>;

impl<T> WebGLSender<T> {
    #[inline]
    pub fn send(&self, data: T) -> Result<(), mpsc::SendError<T>> {
        self.0.send(data)
    }
}

impl<T> WebGLReceiver<T> {
    #[inline]
    pub fn recv(&self) -> Result<T, mpsc::RecvError> {
        self.0.recv()
    }
}

pub fn webgl_channel<T>() -> Result<(WebGLSender<T>, WebGLReceiver<T>), ()> {
    let (sender, receiver) = mpsc::channel();
    Ok((WebGLSender(sender), WebGLReceiver(receiver)))
}

pub trait WebGLSyncPipeline: Send {
    fn channel(&self) -> WebGLChan;
}

pub struct WebGLPipeline(pub Box<WebGLSyncPipeline>);

impl WebGLPipeline {
    pub fn channel(&self) -> WebGLChan {
        self.0.channel()
    }
}

pub trait WebGLSyncCall {
    fn call(&mut self, msg: WebGLMsg, c: &WebGLChan);
}

#[derive(Clone)]
pub struct WebGLChan(pub Rc<RefCell<WebGLSyncCall>>);

impl WebGLChan {
    #[inline]
    pub fn send(&self, msg: WebGLMsg) -> WebGLSendResult {
        Ok(self.0.borrow_mut().call(msg, &self))
    }
}

unreachable_serializable!(WebGLReceiver);
unreachable_serializable!(WebGLSender);

impl Serialize for WebGLChan {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        unreachable!();
    }
}

impl<'a> Deserialize<'a> for WebGLChan {
    fn deserialize<D>(_: D) -> Result<WebGLChan, D::Error>
                    where D: Deserializer<'a> {
        unreachable!();
    }
}


impl Serialize for WebGLPipeline {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        unreachable!();
    }
}

impl<'a> Deserialize<'a> for WebGLPipeline {
    fn deserialize<D>(_: D) -> Result<WebGLPipeline, D::Error>
                    where D: Deserializer<'a> {
        unreachable!();
    }
}
