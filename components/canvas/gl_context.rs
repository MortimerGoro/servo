/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A set of WebGL-related types, in their own module so it's easy to
//! compile it off.

use canvas_traits::webgl::WebGLCommand;
use euclid::Size2D;
use gleam::gl;
use offscreen_gl_context::{NativeGLContext, NativeGLContextHandle};
use offscreen_gl_context::{GLContext, NativeGLContextMethods, GLContextDispatcher};
use offscreen_gl_context::{OSMesaContext, OSMesaContextHandle};
use offscreen_gl_context::{ColorAttachmentType, GLContextAttributes, GLLimits};
use super::webgl_thread::WebGLImpl;

pub enum GLContextFactory {
    Native(NativeGLContextHandle),
    OSMesa(OSMesaContextHandle),
}

impl GLContextFactory {
    pub fn current_native_handle() -> Option<GLContextFactory> {
        NativeGLContext::current_handle().map(GLContextFactory::Native)
    }

    pub fn current_osmesa_handle() -> Option<GLContextFactory> {
        OSMesaContext::current_handle().map(GLContextFactory::OSMesa)
    }

    pub fn new_context(&self,
                       size: Size2D<i32>,
                       attributes: GLContextAttributes,
                       dispatcher: Option<Box<GLContextDispatcher>>) -> Result<GLContextWrapper, &'static str> {
        match *self {
            GLContextFactory::Native(ref handle) => {
                let ctx = GLContext::<NativeGLContext>::new_shared_with_dispatcher(size,
                                                                                   attributes,
                                                                                   ColorAttachmentType::Texture,
                                                                                   gl::GlType::default(),
                                                                                   Some(handle),
                                                                                   dispatcher);
                ctx.map(GLContextWrapper::Native)
            }
            GLContextFactory::OSMesa(ref handle) => {
                let ctx = GLContext::<OSMesaContext>::new_shared_with_dispatcher(size.to_untyped(),
                                                                                 attributes,
                                                                                 ColorAttachmentType::Texture,
                                                                                 gl::GlType::default(),
                                                                                 Some(handle),
                                                                                 dispatcher);
                ctx.map(GLContextWrapper::OSMesa)
            }
        }
    }
}

pub enum GLContextWrapper {
    Native(GLContext<NativeGLContext>),
    OSMesa(GLContext<OSMesaContext>),
}

impl GLContextWrapper {
    pub fn make_current(&self) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                ctx.make_current().unwrap();
            }
            GLContextWrapper::OSMesa(ref ctx) => {
                ctx.make_current().unwrap();
            }
        }
    }

    pub fn unbind(&self) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                ctx.unbind().unwrap();
            }
            GLContextWrapper::OSMesa(ref ctx) => {
                ctx.unbind().unwrap();
            }
        }
    }

    pub fn apply_command(&self, cmd: WebGLCommand) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                WebGLImpl::apply(ctx, cmd);
            }
            GLContextWrapper::OSMesa(ref ctx) => {
                WebGLImpl::apply(ctx, cmd);
            }
        }
    }

    pub fn get_info(&self) -> (Size2D<i32>, u32, GLLimits) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                let (real_size, texture_id) = {
                    let draw_buffer = ctx.borrow_draw_buffer().unwrap();
                    (draw_buffer.size(), draw_buffer.get_bound_texture_id().unwrap())
                };

                let limits = ctx.borrow_limits().clone();

                (real_size, texture_id, limits)
            }
            GLContextWrapper::OSMesa(ref ctx) => {
                let (real_size, texture_id) = {
                    let draw_buffer = ctx.borrow_draw_buffer().unwrap();
                    (draw_buffer.size(), draw_buffer.get_bound_texture_id().unwrap())
                };

                let limits = ctx.borrow_limits().clone();

                (real_size, texture_id, limits)
            }
        }
    }

    pub fn resize(&mut self, size: Size2D<i32>) -> Result<(), &'static str> {
        match *self {
            GLContextWrapper::Native(ref mut ctx) => {
                ctx.resize(size)
            }
            GLContextWrapper::OSMesa(ref mut ctx) => {
                ctx.resize(size)
            }
        }
    }
}
