/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvr/#interface-vrstageparameters
[NoInterfaceObject]
interface VRView {
  readonly attribute WebGLFramebuffer framebuffer;
  VRViewport getViewport();
  VRAttributes getAttributes();
};

dictionary VRViewport {
  long x;
  long y;
  long width;
  long height;
};

dictionary VRAttributes {
  boolean depth = false;
  boolean multiview = false;
  boolean antialias = false;
};
