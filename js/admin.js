function enabledClicked(checkbox) {

    // TODO: disable control until a response is received
    // TODO: show a spinner

    fetch('/api/admin/stream', {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                enabled: checkbox.checked
            }),
            credentials: 'same-origin'
        })
        .then(response => {
            // TODO: re-enable control
            // TODO: hide spinner
            console.log(response)
        })
}
