package com.mozilla.servo;

import android.Manifest;
import android.app.Activity;
import android.content.Context;
import android.content.pm.PackageManager;
import android.graphics.SurfaceTexture;
import android.hardware.Camera;
import android.hardware.Camera.Parameters;
import android.util.Log;
import android.support.v4.app.ActivityCompat;
import android.support.v4.content.ContextCompat;

import java.io.IOException;

public class VRCamera {
    private static String TAG = "VRCamera";
    private SurfaceTexture mSurfaceTexture;
    private boolean mPaused = false;
    private Camera camera;
    private boolean cameraSetUpStatus;
    private int fpsMode = -1;
    private boolean isCameraOpen = false;

    public VRCamera(int textureId) {
        isCameraOpen = true;
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
            camera.startPreview();
            camera.setPreviewTexture(mSurfaceTexture);
            isCameraOpen = true;
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
        isCameraOpen = false;
    }

    public void resume() {
        if (openCamera()) {
            //restore fpsmode
            setUpCameraForVrMode(1);
        }
        mPaused = false;
    }


    public void pause() {
        mPaused = true;
        closeCamera();
    }

    public void close() {
        closeCamera();
    }

    public void update() {
        if (!mPaused) {
            // Update the latest camera frame.
            // If there isn't anything new, it reuses whatever was there before.
            mSurfaceTexture.updateTexImage();
        }
    }

    private boolean checkCameraHardware(Context context) {
        return context.getPackageManager().hasSystemFeature(
                PackageManager.FEATURE_CAMERA);
    }

    /**
     * Configure high fps settings in the camera for VR mode
     *
     * @param fpsMode integer indicating the desired fps: 0 means 30 fps, 1 means 60
     *                fps, and 2 means 120 fps. Any other value is invalid.
     * @return A boolean indicating the status of the method call. It may be false due
     * to multiple reasons including: 1) supplying invalid fpsMode as the input
     * parameter, 2) VR mode not supported.
     */
    public boolean setUpCameraForVrMode(final int fpsMode) {
        cameraSetUpStatus = false;
        this.fpsMode = fpsMode;

        if (!isCameraOpen) {
            Log.e(TAG, "Camera is not open");
            return false;
        }
        if (fpsMode < 0 || fpsMode > 2) {
            //Log.e(TAG,  "Invalid fpsMode: %d. It can only take values 0, 1, or 2.", fpsMode);
        } else {
            Parameters params = camera.getParameters();

            // check if the device supports vr mode preview
            if ("true".equalsIgnoreCase(params.get("vrmode-supported"))) {

                Log.v(TAG, "VR Mode supported!");

                // set vr mode
                params.set("vrmode", 1);

                // true if the apps intend to record videos using
                // MediaRecorder
                params.setRecordingHint(true);

                // set preview size
                // params.setPreviewSize(640, 480);

                // set fast-fps-mode: 0 for 30fps, 1 for 60 fps,
                // 2 for 120 fps
                params.set("fast-fps-mode", fpsMode);

                switch (fpsMode) {
                    case 0: // 30 fps
                        params.setPreviewFpsRange(30000, 30000);
                        break;
                    case 1: // 60 fps
                        params.setPreviewFpsRange(60000, 60000);
                        break;
                    case 2: // 120 fps
                        params.setPreviewFpsRange(120000, 120000);
                        break;
                    default:
                }

                // for auto focus
                params.set("focus-mode", "continuous-video");

                params.setVideoStabilization(false);
                if ("true".equalsIgnoreCase(params.get("ois-supported"))) {
                    params.set("ois", "center");
                }

                camera.setParameters(params);
                cameraSetUpStatus = true;
            }
        }

        return cameraSetUpStatus;
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
