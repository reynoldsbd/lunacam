function enabledClicked(checkbox) {

    // TODO: disable control until a response is received

    if (checkbox.checked) {
        body = '{"enabled": true}'
    } else {
        body = '{"enabled": false}'
    }

    fetch('/api/admin/stream', {
            method: 'POST',
            body: body,
            credentials: 'same-origin'
        })
        .then(response => {
            console.log(response)
        })
}
