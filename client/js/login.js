
var submitButton = document.getElementById('submit');
var usernameField = document.getElementById('username');
var passwordField = document.getElementById('password');

var activeSubmission = false;

function handleSubmitResponse(response) {

    activeSubmission = false;
    submitButton.disabled = false;
    submitButton.classList.remove('is-loading');

    if (response.ok) {

        let dest = window.location.origin;
        let searchParams = new URLSearchParams(window.location.search);

        if (searchParams.has('dest')) {
            dest += searchParams.get('dest');
        } else {
            dest += '/';
        }

        window.location = dest;
    } else {

        // TODO: extract message from response
        showMessage('Failed to login', 'error');
    }
}

function submit() {

    // Only allow one in-flight submission at a time
    if (activeSubmission) {
        return;
    }

    activeSubmission = true;
    submitButton.disabled = true;
    submitButton.classList.add('is-loading');

    let url = '/api/sessions';
    let init = {
        headers: {
            'Content-Type': 'application/json'
        },
        credentials: 'same-origin',
        method: 'PUT',
        body: JSON.stringify({
            username: usernameField.value,
            password: passwordField.value
        })
    };

    fetch(url, init)
        .then(handleSubmitResponse);
}

submitButton.onclick = submit;
