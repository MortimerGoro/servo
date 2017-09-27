package com.mozilla.servo;

import android.Manifest;
import android.app.Activity;
import android.content.Context;
import android.content.pm.PackageManager;
import android.graphics.Rect;
import android.graphics.SurfaceTexture;
import android.hardware.Camera;
import android.hardware.Camera.Parameters;
import android.util.Log;
import android.support.v4.app.ActivityCompat;
import android.support.v4.content.ContextCompat;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

public class VRCamera {
    private static String TAG = "VRCamera";
    private SurfaceTexture mSurfaceTexture;
    private boolean mPaused = false;
    private Camera camera;

    public VRCamera(int textureId) {
        mSurfaceTexture = new SurfaceTexture(textureId);
    }

    private boolean openCamera() {
        if (camera != null) {
            //already open
            return true;
        }
        try {
            camera = Camera.open();

            if (camera == null) {
                android.util.Log.d(TAG, "Camera not available or is in use");
                return false;
            }
            int w = 1920;
            int h = 1080;
            mSurfaceTexture.setDefaultBufferSize(w, h);
            setupCameraParameters(w, h, FPSMode.FPS60);
            camera.setPreviewTexture(mSurfaceTexture);
            camera.startPreview();
        } catch (Exception exception) {
            android.util.Log.d(TAG, "Camera not available or is in use");
            return false;
        }

        return true;
    }

    private void closeCamera() {
        if (camera == null) {
            //nothing to do
            return;
        }

        camera.stopPreview();
        camera.release();
        camera = null;
    }

    public void resume() {
        openCamera();
        mPaused = false;
    }

    public void pause() {
        mPaused = true;
        closeCamera();
    }

    public void close() {
        closeCamera();
    }

    public long update() {
        if (!mPaused) {
            // Update the latest camera frame.
            // If there isn't anything new, it reuses whatever was there before.
            mSurfaceTexture.updateTexImage();
            return mSurfaceTexture.getTimestamp();
        }
        return 0;
    }

    private boolean checkCameraHardware(Context context) {
        return context.getPackageManager().hasSystemFeature(
                PackageManager.FEATURE_CAMERA);
    }

    private enum FPSMode {
        FPS30,
        FPS60,
        FPS120,
    }

    // Specific optimizations supported for some VR devices (e.g. Gear VR)
    private void setupCameraParameters(int w, int h, FPSMode fpsMode) {
        Parameters params = camera.getParameters();
        //List<Camera.Size> sizes = camera.getParameters().getSupportedVideoSizes();
        params.setPreviewSize(w, h);
        // We don't want to record
        params.setRecordingHint(false);
        // for auto focus
        params.setFocusMode(Parameters.FOCUS_MODE_INFINITY);
        /*ArrayList<Camera.Area> areas = new ArrayList<>();
        int focusW = 200;
        int focusH = 200;
        areas.add(new Camera.Area(new Rect(w/2 - focusW/2, h/2 - focusH/2, w/2 + focusW/2, h/2 + focusH/2), 1000));
        params.setFocusAreas(areas);
        params.setMeteringAreas(areas);*/
        params.setVideoStabilization(false);

        params.set("fast-fps-mode", fpsMode.ordinal());

        switch (fpsMode) {
            case FPS30: // 30 fps
                params.setPreviewFpsRange(30000, 30000);
                break;
            case FPS60: // 60 fps
                params.setPreviewFpsRange(60000, 60000);
                break;
            case FPS120: // 120 fps
                params.setPreviewFpsRange(120000, 120000);
                break;
            default:
        }

        // Optical image stabilization
        if ("true".equalsIgnoreCase(params.get("ois-supported"))) {
            params.set("ois", "center");
        }

        // check if the device supports vr mode preview
        if ("true".equalsIgnoreCase(params.get("vrmode-supported"))) {
            Log.v(TAG, "VR Mode supported!");
            // set vr mode
            params.set("vrmode", 1);
        }

        camera.setParameters(params);
    }

    private static final String CAMERA_PERMISSION = Manifest.permission.CAMERA;
    private static final int CAMERA_PERMISSION_CODE = 0;

    public static boolean hasCameraPermission(Activity activity) {
        return ContextCompat.checkSelfPermission(activity, CAMERA_PERMISSION) ==
                PackageManager.PERMISSION_GRANTED;
    }

    public static void requestCameraPermission(Activity activity) {
        ActivityCompat.requestPermissions(activity, new String[]{CAMERA_PERMISSION},
                CAMERA_PERMISSION_CODE);
    }

}
