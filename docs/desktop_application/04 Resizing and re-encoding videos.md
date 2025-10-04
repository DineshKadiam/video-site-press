<!--
    SPDX-FileCopyrightText: 2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Resizing and re-encoding videos

If you want to ...

- resize the video (e.g. because you have a 4k video and want to provide 1080p and 720p versions instead)
- re-encode the video (e.g. because you have provided a lossless version of the video, unsuitable for streaming)
- transcode the video (e.g. because Hyper 8 tells you that your video is not compatible with browsers)

... you can go to the `Encode video` section on the `Video/Audio files`
subpage of your video to request additional versions of your video:

1. Choose a resolution from the drop-down selector – this is the only decision that
   you always need to make.

2. If you don't care about further details you can already click the `Render
   Format` button and skip point 3. – Hyper 8 will pick reasonable defaults for everything else.

3. If you want to fine-tune the outcome, open the `Advanced settings` toggle and adjust:

    - **Frame Rate** – This should usually only be changed if your original video file is (for instance) using 60 frames per second, but you want to half the storage and bandwith use by reducing it to (entirely sufficient) 30 fps

    - **Quality** – Experiment with lower values to lower your storage and bandwith use. Usually you can get away with much lower quality settings than the default. Consider that your audience will prefer fast-loading videos with minor compression flaws over pixel-perfect videos that take forever to load!

    - **Codec** – At this point it is not (yet) recommended to use anything else than H.264 - VP9 will be a viable option sometime around 2026/2027 but right now it is not yet as ubiquitously supported.

    - **Encoding approach** – For anything but temporary experiments, the `Highest compression` setting should be your choice - it takes long to process, but in the long run the storage/bandwidth savings are worth it

Sometimes you want to limit certain video files to certain uses, e.g. to
provide high quality versions only for download, low quality versions only
for streaming, or to keep the original file(s) offline, using them only as a
source for video processing. This is covered on the next page.
