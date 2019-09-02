var stream = document.getElementById('stream');

window.onload = function() {
    
    if (Hls.isSupported()) {
        let hls = new Hls();
        hls.loadSource(stream.dataset.streamUrl);
        hls.attachMedia(stream);
        hls.on(Hls.Events.MANIFEST_PARSED, function() { stream.play(); });

    } else if (stream.canPlayType('application/vnd.apple.mpegurl')) {
        stream.src = stream.dataset.streamUrl;
        stream.addEventListener('loadmetadata', function() { stream.play(); });

    } else {
        showMessage('Browser does not support HLS streaming', 'warning');
    }
}
