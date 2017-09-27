/* global mat4, WGLUProgram */
window.VRCubeCamera = (function () {
    "use strict";
  
    var cubeCameraVS = [
      "uniform mat4 projectionMat;",
      "uniform mat4 modelViewMat;",
      "attribute vec3 position;",
      "attribute vec2 texCoord;",
      "varying vec2 vTexCoord;",
  
      "void main() {",
      "  vTexCoord = texCoord;",
      "  gl_Position = projectionMat * modelViewMat * vec4( position, 1.0 );",
      "}",
    ].join("\n");
  
    var cubeCameraFS = [
      "precision mediump float;",
      "uniform sampler2D diffuse;",
      "varying vec2 vTexCoord;",

      "void main() {",
      "  gl_FragColor = texture2D(diffuse, vTexCoord);",
      "}",
    ].join("\n");

    var cubeCameraFS_OES = [
        "#extension GL_OES_EGL_image_external : require",
        "precision mediump float;",
        "uniform samplerExternalOES diffuse;",
        "varying vec2 vTexCoord;",
  
        "void main() {",
        "  gl_FragColor = texture2D(diffuse, vTexCoord);",
        "}",
      ].join("\n");
  
    var CubeCamera = function (gl, texture, width, height) {
      this.gl = gl;
  
      var fs = cubeCameraFS;
      this.texture = texture;

      if (gl.texImageCamera) {
          fs = cubeCameraFS_OES;
          this.texture = gl.createTexture();
          gl.texImageCamera(this.texture);
      }
  
      this.program = new WGLUProgram(gl);
      this.program.attachShaderSource(cubeCameraVS, gl.VERTEX_SHADER);
      this.program.attachShaderSource(fs, gl.FRAGMENT_SHADER);
      this.program.bindAttribLocation({
        position: 0,
        texCoord: 1
      });
      this.program.link();
  
      this.vertBuffer = gl.createBuffer();
      this.indexBuffer = gl.createBuffer();

      this.orthoProjMatrix = mat4.create();
      this.orthoViewMatrix = mat4.create();
  
      this.resize(width, height);
    };
  
    CubeCamera.prototype.resize = function (width, height) {
      var gl = this.gl;

      //mat4.ortho(this.orthoProjMatrix, 0, width, 0, height, 0.1, 1024);
      mat4.identity(this.orthoProjMatrix)
      mat4.identity(this.orthoViewMatrix);
  
      this.width = width;
      this.height = height;
  
      var vertices = [];
      var indices = [];

      var z = 100;
      var w = 150;
      var h = w * height/width;
  
      // Build a single quad.
      function appendQuad (left, bottom, right, top) {
        // Bottom
        indices.push(0, 1, 2, 3, 2, 1);
  
        vertices.push(left, bottom, -z,  0.0, 1.0);
        vertices.push(right, bottom, -z, 1.0, 1.0);
        vertices.push(left, top, -z, 0.0, 0.0);
        vertices.push(right, top, -z, 1.0, 0.0);
      }

      appendQuad(-w, -h, w, h);
  
      gl.bindBuffer(gl.ARRAY_BUFFER, this.vertBuffer);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.STATIC_DRAW);
  
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), gl.STATIC_DRAW);
  
      this.indexCount = indices.length;
    };
  
    CubeCamera.prototype.render = function (projection) {
      var gl = this.gl;
      var program = this.program;
  
      program.use();
  
      gl.uniformMatrix4fv(program.uniform.projectionMat, false, projection);
      gl.uniformMatrix4fv(program.uniform.modelViewMat, false, this.orthoViewMatrix);
  
      gl.bindBuffer(gl.ARRAY_BUFFER, this.vertBuffer);
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);
  
      gl.enableVertexAttribArray(program.attrib.position);
      gl.enableVertexAttribArray(program.attrib.texCoord);
  
      gl.vertexAttribPointer(program.attrib.position, 3, gl.FLOAT, false, 20, 0);
      gl.vertexAttribPointer(program.attrib.texCoord, 2, gl.FLOAT, false, 20, 12);
      if (gl.texImageCameraUpdate) {
        gl.activeTexture(gl.TEXTURE1);
        gl.uniform1i(this.program.uniform.diffuse, 1);
        gl.texImageCameraUpdate(this.texture);
        gl.drawElements(gl.TRIANGLES, this.indexCount, gl.UNSIGNED_SHORT, 0);
        gl.activeTexture(gl.TEXTURE0);
        gl.clear(gl.DEPTH_BUFFER_BIT);
      }
      else {
        gl.activeTexture(gl.TEXTURE0);
        gl.uniform1i(this.program.uniform.diffuse, 0);
        gl.bindTexture(gl.TEXTURE_2D, this.texture);
        gl.drawElements(gl.TRIANGLES, this.indexCount, gl.UNSIGNED_SHORT, 0);
      }

      gl.flush();

    };
  
    return CubeCamera;
  })();
  