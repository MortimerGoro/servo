/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{WebGLBufferId, WebGLCommand, WebGLError, WebGLMsgSender, WebGLResult, WebGLVertexArrayId};
use canvas_traits::webgl::webgl_channel;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLBufferBinding;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::webglobject::WebGLObject;
use dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::collections::HashSet;


#[dom_struct]
pub struct WebGLBuffer {
    webgl_object: WebGLObject,
    id: WebGLBufferId,
    /// The target to which this buffer was bound the first time
    target: Cell<Option<u32>>,
    capacity: Cell<usize>,
    is_deleted: Cell<bool>,
    currently_bound: Cell<bool>,
    // The Vertex Array Objects that are referencing this buffer
    vao_references: DomRefCell<Option<HashSet<WebGLVertexArrayId>>>,
    pending_delete: Cell<bool>,
    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    renderer: WebGLMsgSender,
}

impl WebGLBuffer {
    fn new_inherited(renderer: WebGLMsgSender,
                     id: WebGLBufferId)
                     -> WebGLBuffer {
        WebGLBuffer {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            target: Cell::new(None),
            capacity: Cell::new(0),
            is_deleted: Cell::new(false),
            currently_bound: Cell::new(false),
            vao_references: DomRefCell::new(None),
            pending_delete: Cell::new(false),
            renderer: renderer,
        }
    }

    pub fn maybe_new(window: &Window, renderer: WebGLMsgSender)
                     -> Option<DomRoot<WebGLBuffer>> {
        let (sender, receiver) = webgl_channel().unwrap();
        renderer.send(WebGLCommand::CreateBuffer(sender)).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|buffer_id| WebGLBuffer::new(window, renderer, buffer_id))
    }

    pub fn new(window: &Window,
               renderer: WebGLMsgSender,
               id: WebGLBufferId)
              -> DomRoot<WebGLBuffer> {
        reflect_dom_object(Box::new(WebGLBuffer::new_inherited(renderer, id)),
                           window, WebGLBufferBinding::Wrap)
    }
}


impl WebGLBuffer {
    pub fn id(&self) -> WebGLBufferId {
        self.id
    }

    fn is_bind_allowed(&self, first: u32, second: u32) -> bool {
        // Rebinding any target to COPY target is allowed.
        if second == constants::COPY_READ_BUFFER || second == constants::COPY_WRITE_BUFFER {
            return true;
        }

        if self.is_currently_bound() && first != second &&
           (first == constants::TRANSFORM_FEEDBACK_BUFFER || second == constants::TRANSFORM_FEEDBACK_BUFFER ) {
            // TRANSFORM_FEEDBACK_BUFFER can't be simultaneously bound with another target.
            return false;
        }

        // Rebinding ELEMENT_ARRAY_BUFFER to/from other target types is forbidden.
        (first == constants::ELEMENT_ARRAY_BUFFER) == (second == constants::ELEMENT_ARRAY_BUFFER)
    }

    pub fn update_target(&self, target: u32) -> WebGLResult<()> {
        if self.target.get().map(|prev_target| self.is_bind_allowed(prev_target, target)) == Some(false) {
            return Err(WebGLError::InvalidOperation);
        }
        self.target.set(Some(target));
        Ok(())
    }

    // NB: Only valid buffer targets come here
    pub fn bind(&self, target: u32) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        self.update_target(target)?;
        let msg = WebGLCommand::BindBuffer(target, Some(self.id));
        self.renderer.send(msg).unwrap();

        Ok(())
    }

    pub fn buffer_data(&self, target: u32, data: &[u8], usage: u32) -> WebGLResult<()> {
        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        }
        self.capacity.set(data.len());
        self.renderer.send(WebGLCommand::BufferData(target, data.to_vec(), usage)).unwrap();

        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(WebGLCommand::DeleteBuffer(self.id));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn is_currently_bound(&self) -> bool {
        self.currently_bound.get()
    }

    pub fn set_currently_bound(&self, value: bool) {
        self.currently_bound.set(value);
    }


    pub fn target(&self) -> Option<u32> {
        self.target.get()
    }

    pub fn is_attached_to_vao(&self) -> bool {
        self.vao_references.borrow().as_ref().map_or(false, |vaos| !vaos.is_empty())
    }

    pub fn set_pending_delete(&self) {
        self.pending_delete.set(true);
    }

    pub fn add_vao_reference(&self, id: WebGLVertexArrayId) {
        let mut vao_refs = self.vao_references.borrow_mut();
        if let Some(ref mut vao_refs) = *vao_refs {
            vao_refs.insert(id);
            return;
        }

        let mut map = HashSet::new();
        map.insert(id);
        *vao_refs = Some(map);
    }

    pub fn remove_vao_reference(&self, id: WebGLVertexArrayId) {
        if let Some(ref mut vao_refs) = *self.vao_references.borrow_mut() {
            if vao_refs.take(&id).is_some() && self.pending_delete.get() {
                // WebGL spec: The deleted buffers should no longer be valid when the VAOs are deleted
                let _ = self.renderer.send(WebGLCommand::DeleteBuffer(self.id));
                self.is_deleted.set(true);
            }
        }
    }

    /// Updates the slot user for storing bound buffer targets and ensures that currently_bound
    /// flag is correctly set for both bound and unbound buffers.
    pub fn update_slot(slot: &MutNullableDom<WebGLBuffer>, buffer: Option<&WebGLBuffer>) {
        if let Some(b) = slot.get() {
            b.set_currently_bound(false);
        }
        if let Some(b) = buffer {
            b.set_currently_bound(true);
        }
        slot.set(buffer);
    }
}

impl Drop for WebGLBuffer {
    fn drop(&mut self) {
        self.delete();
    }
}
