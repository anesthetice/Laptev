#!/usr/bin/python3
# I did not shamelessly steal this code from https://github.com/raspberrypi/picamera2/blob/af75efd3b76479a2f6401dda120a67f3ee417eea/examples/capture_motion.py
# and adapt it to my needs as I am a good programmer

from time import time
from numpy import square, subtract
from picamera2 import Picamera2
from picamera2.encoders import H264Encoder
from picamera2.outputs import FileOutput
from libcamera import controls

lsize = (400, 300)
picam2 = Picamera2()

# AEC algorithm settings
picam2.set_controls({"AeConstraintMode": controls.AeConstraintModeEnum.Normal})
picam2.set_controls({"AeEnable": True})

# Autofocus settings
picam2.set_controls({"AfMode": controls.AfModeEnum.Continuous})
picam2.set_controls({"AfRange": controls.AfRangeEnum.Normal})
picam2.set_controls({"AfSpeed": controls.AfSpeedEnum.Fast})

# Other settings
picam2.set_controls({"NoiseReductionMode": controls.draft.NoiseReductionModeEnum.HighQuality})
picam2.set_controls({"AwbEnable": True})
picam2.set_controls({"AwbMode": controls.AwbModeEnum.Indoor})
picam2.set_controls({"Brightness": 0.15}) # max : 1.0
picam2.set_controls({"Contrast": 1.4}) # max : 32.0
picam2.set_controls({"Sharpness": 2.0}) # max : 16.0

"""
currently disabled, see line 64
# capture config for the still images that will be used for face detection and recognition
capture_config = picam2.create_still_configuration()
"""

# H264 seemingly doesn't work with 2304x1296, so I'll be using the standard 1080p 16:9 format
video_config = picam2.create_video_configuration(main={"size": (1920, 1080), "format": "RGB888"},
                                                 lores={"size": lsize, "format": "YUV420"})
picam2.configure(video_config)
# high quality 1080p bitrate
encoder = H264Encoder(162560000)
picam2.encoder = encoder
picam2.start()

w, h = lsize
prev = None
encoding = False
ltime = 0

motion_count = 0
motion_elapsed_counter = 0

while True:
    cur = picam2.capture_buffer("lores")
    cur = cur[:w * h].reshape(h, w)
    if prev is not None:
        # Measure pixels differences between current and
        # previous frame
        mse = square(subtract(cur, prev)).mean()
        if mse > 5:
            motion_count += 1
            motion_elapsed_counter = 0
            """
            disabled as it's not very useful, might need to reimplement it in the future
            if motion_count == 2:
                print("motion probably detected, taking images")
                picam2.switch_mode(capture_config)
                picam2.capture_file(f"output/image-{int(time())}.jpg")
                sleep(0.1)
                picam2.capture_file(f"output/image-{int(time())}.jpg")
                sleep(0.1)
                picam2.capture_file(f"output/image-{int(time())}.jpg")
                picam2.switch_mode(video_config)
                picam2.encoder = encoder
            """
            if motion_count >= 3:
                if not encoding:
                    print("motion detected", mse)
                    encoder.output = FileOutput(f"output/{int(time())}.h264")
                    picam2.start_encoder()
                    encoding = True
                ltime = time()
        else:
            if encoding and time() - ltime > 1.5:
                picam2.stop_encoder()
                encoding = False
                motion_count = 0
        if motion_count > 0:
            motion_elapsed_counter += 1
        if motion_elapsed_counter > 50:
            motion_count = 0
    prev = cur
