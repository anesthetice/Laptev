#!/usr/bin/python3

from libcamera import controls
from numpy import square, subtract
from picamera2 import Picamera2
from picamera2.encoders import H264Encoder, Quality
from picamera2.outputs import FfmpegOutput
from PIL import Image
from time import time, sleep

lq_size = (576, 324)
hq_size = (1536, 864)
picam2 = Picamera2()

video_config = picam2.create_video_configuration(
    main={"size": hq_size, "format": "RGB888"},
    lores={"size": lq_size, "format": "YUV420"},
    controls={"FrameRate": 60.0}
)
picam2.configure(video_config)

# AEC algorithm settings
picam2.set_controls({"AeConstraintMode": controls.AeConstraintModeEnum.Normal})
picam2.set_controls({"AeEnable": True})

# Autofocus settings
picam2.set_controls({"AfMode": controls.AfModeEnum.Continuous})
picam2.set_controls({"AfRange": controls.AfRangeEnum.Normal})
picam2.set_controls({"AfSpeed": controls.AfSpeedEnum.Fast})

# Other settings
picam2.set_controls({"AwbEnable": True})
picam2.set_controls({"AwbMode": controls.AwbModeEnum.Indoor})
picam2.set_controls({"Brightness": 0.20}) # max : 1.0
picam2.set_controls({"Contrast": 1.0}) # max : 32.0
picam2.set_controls({"Sharpness": 1.0}) # max : 16.0

encoder = H264Encoder()
picam2.encoder = encoder
picam2.start()

def get_mse_threshold(samples=100):
    w, h = lq_size
    prev = None
    sum = 0
    for _ in range (0, samples):
        cur = picam2.capture_buffer("lores")
        cur = cur[:w * h].reshape(h, w)
        if prev is not None:
            # Measure pixels differences between current and
            # previous frame
            mse = square(subtract(cur, prev)).mean()
            sum += mse
        prev = cur
    sum = sum/samples + 4
    print(f"new threshold: {sum}")
    return sum

threshold = get_mse_threshold()
threshold_update_guard = 0
w, h = lq_size
prev = None
encoding = False
ltime = 0
itime = 0
cur_time = 0

motion_count = 0
motion_elapsed_counter = 0
lazy_counter = 0


while True:
    cur = picam2.capture_buffer("lores")
    cur = cur[:w * h].reshape(h, w)
    if prev is not None:
        # Measure pixels differences between current and
        # previous frame
        cur_time = time()
        mse = square(subtract(cur, prev)).mean()
        if threshold_update_guard > 100:
            print("threshold guard reached")
            threshold = get_mse_threshold()
            threshold_update_guard = 0

        if mse > threshold:
            motion_count += 1
            motion_elapsed_counter = 0

            if motion_count >= 3:
                if not encoding:
                    print("motion detected, started encoding")
                    itime = cur_time
                    timestamp = int(itime)

                    thumbnail = Image.fromarray(picam2.capture_array("main"), "RGB")
                    thumbnail.thumbnail((512, 288))
                    thumbnail.save(f"data/{timestamp}.jpg")

                    encoder.output = FfmpegOutput(f"data/{timestamp}.mp4")
                    picam2.start_encoder(encoder=picam2.encoder, output=encoder.output, quality=Quality.LOW)
                    encoding = True
                    motion_count = 0
                    threshold_update_guard += 2
                ltime = cur_time
            if cur_time - itime > 10.0:
                if encoding: 
                    threshold_update_guard += 20
                    print("10 seconds reached, stopped encoding")
                    picam2.stop_encoder()
                    encoding = False
                    motion_count = 0
                
        else:
            timediff = cur_time - ltime
            if encoding and timediff > 2.25:
                print(f"stopped encoding, {timediff}")
                picam2.stop_encoder()
                encoding = False
                motion_count = 0
    prev = cur

