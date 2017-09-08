#!/bin/bash

# Stop
adb shell am force-stop com.mozilla.servo.multiview
# Run
adb shell am start -a android.intent.action.VIEW -d "https://threejs.org/examples/webvr_rollercoaster.html" com.mozilla.servo.multiview/com.mozilla.servo.MainActivity
#adb shell am start -a android.intent.action.VIEW -d "http://192.168.0.11:8000/room-scale.html" com.mozilla.servo.multiview/com.mozilla.servo.MainActivity
