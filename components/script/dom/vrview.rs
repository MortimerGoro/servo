/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{WebGLFramebufferId, WebGLMsg, WebGLMsgSender, WebVRCommand, WebVRDeviceId};
use dom::bindings::codegen::Bindings::VRViewBinding;
use dom::bindings::codegen::Bindings::VRViewBinding::{VRAttributes, VRViewMethods, VRViewport};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{MutNullableDom, DomRoot};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use dom::webglframebuffer::{OpaqueFBOMessages, WebGLFramebuffer};
use webvr_traits::WebVRFramebuffer;

#[dom_struct]
pub struct VRView {
    reflector_: Reflector,
    renderer: WebGLMsgSender,
    device_id: WebVRDeviceId,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    fbo: WebVRFramebuffer,
    webgl_fbo: MutNullableDom<WebGLFramebuffer>,
}

impl VRView {
    fn new_inherited(renderer: WebGLMsgSender,
                     device_id: WebVRDeviceId,
                     fbo: WebVRFramebuffer) -> VRView {
        VRView {
            reflector_: Reflector::new(),
            renderer,
            device_id,
            fbo,
            webgl_fbo: Default::default(),
        }
    }

    pub fn new(global: &GlobalScope,
               renderer: WebGLMsgSender,
               device_id: WebVRDeviceId,
               fbo: WebVRFramebuffer) -> DomRoot<VRView> {
        reflect_dom_object(Box::new(VRView::new_inherited(renderer, device_id, fbo)),
                           global,
                           VRViewBinding::Wrap)
    }
}

impl VRViewMethods for VRView {
    // https://w3c.github.io/webvr/#interface-interface-vrfieldofview
    #[allow(unsafe_code)]
    fn Framebuffer(&self) -> DomRoot<WebGLFramebuffer> {
        self.webgl_fbo.or_init(|| {
            let fbo_id = unsafe {
                // Generate a dummy FBO id that avoids collisions with the real WebGL FBOs.
                WebGLFramebufferId::new(self.device_id * 1000 + self.fbo.eye_index)
            };
            let bind_msg = WebGLMsg::WebVRCommand(self.renderer.context_id(),
                                                  WebVRCommand::BindFramebuffer(self.device_id,
                                                                                self.fbo.eye_index));
            let unbind_msg = Some(WebGLMsg::WebVRCommand(self.renderer.context_id(),
                                                         WebVRCommand::UnbindFramebuffer(self.device_id,
                                                                                         self.fbo.eye_index)));
            let opaque_messages = OpaqueFBOMessages {
                bind_msg, unbind_msg
            };

            WebGLFramebuffer::new_opaque(self.global().as_window(),
                                         self.renderer.clone(),
                                         fbo_id,
                                         opaque_messages)
        })
    }

    fn GetViewport(&self) -> VRViewport {
        VRViewport {
            x: Some(self.fbo.viewport.x),
            y: Some(self.fbo.viewport.y),
            width: Some(self.fbo.viewport.width),
            height: Some(self.fbo.viewport.height),
        }
    }

    fn GetAttributes(&self) -> VRAttributes {
        VRAttributes {
            depth: self.fbo.attributes.depth,
            multiview: self.fbo.attributes.multiview,
            antialias: self.fbo.attributes.multisampling,
        }
    }
}
