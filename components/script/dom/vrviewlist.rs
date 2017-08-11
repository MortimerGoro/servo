/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::VRViewListBinding;
use dom::bindings::codegen::Bindings::VRViewListBinding::VRViewListMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::vrview::VRView;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct VRViewList {
    reflector_: Reflector,
    views: Vec<JS<VRView>>,
}

impl VRViewList {
    fn new_inherited(views: &[&VRView]) -> VRViewList {
        VRViewList {
            reflector_: Reflector::new(),
            views: views.iter().map(|VRView| JS::from_ref(*VRView)).collect(),
        }
    }

    pub fn new(window: &Window, views: &[&VRView]) -> Root<VRViewList> {
        reflect_dom_object(box VRViewList::new_inherited(views),
                           window, VRViewListBinding::Wrap)
    }
}

impl VRViewListMethods for VRViewList {
    /// https://w3c.github.io/VRView-events/#widl-VRViewList-length
    fn Length(&self) -> u32 {
        self.views.len() as u32
    }

    /// https://w3c.github.io/VRView-events/#widl-VRViewList-item-getter-VRView-unsigned-long-index
    fn Item(&self, index: u32) -> Option<Root<VRView>> {
        self.views.get(index as usize).map(|js| Root::from_ref(&**js))
    }

    /// https://w3c.github.io/VRView-events/#widl-VRViewList-item-getter-VRView-unsigned-long-index
    fn IndexedGetter(&self, index: u32) -> Option<Root<VRView>> {
        self.Item(index)
    }
}
