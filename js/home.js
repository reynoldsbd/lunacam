// Behavior for the home page


//#region HLS Streaming

function loadStream() {
    var video = document.getElementById('video');
    if (Hls.isSupported()) {
        var hls = new Hls();
        hls.loadSource('/stream/video0.m3u8');
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, function() { video.play(); });
    }
    else if (video.canPlayType('application/vnd.apple.mpegurl')) {
        video.src = '/stream/video0.m3u8';
        video.addEventListener('loadmetadata', function() { video.play(); });
    }
}

//#endregion


window.onload = function() {
    loadStream();
}
