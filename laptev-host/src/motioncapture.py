#!/usr/bin/python3
# I did not shamelessly steal this code from https://github.com/raspberrypi/picamera2/blob/af75efd3b76479a2f6401dda120a67f3ee417eea/examples/capture_motion.py
# and adapt it to my needs as I am a good programmer

from libcamera import controls
from numpy import square, subtract
from picamera2 import Picamera2
from picamera2.encoders import H264Encoder
from picamera2.outputs import FileOutput
from PIL import Image
from time import time

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
picam2.set_controls({"Brightness": 0.20}) # max : 1.0
picam2.set_controls({"Contrast": 1.5}) # max : 32.0
picam2.set_controls({"Sharpness": 2.5}) # max : 16.0

# H264 seemingly doesn't work with 2304x1296, so I'll be using the standard 1080p 16:9 format
video_config = picam2.create_video_configuration(
    main={"size": (1920, 1080), "format": "RGB888"},
    lores={"size": lsize, "format": "YUV420"}
)
picam2.configure(video_config)

# decent quality 1080p H264 bitrate (2**27)
encoder = H264Encoder(134217728)
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
        if mse > 6.25:
            motion_count += 1
            motion_elapsed_counter = 0

            if motion_count >= 3:
                if not encoding:
                    timestamp = int(time())
                    print(f"[{timestamp}] motion detected")

                    thumbnail = Image.fromarray(picam2.capture_array("main"))
                    thumbnail.thumbnail((640, 360))
                    thumbnail.save(f"data/{timestamp}.jpg")

                    encoder.output = FileOutput(f"data/{timestamp}.h264")
                    picam2.start_encoder(picam2.encoder, encoder.output)
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