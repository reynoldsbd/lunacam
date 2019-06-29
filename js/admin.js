// Behavior for the admin page


//#region Stream Configuration

var streamEnabledSwitch = document.getElementById('stream-enabled');

function loadStreamState() {
    fetch('/api/admin/stream', {
            method: 'GET',
            credentials: 'same-origin'
        })
        .then(response => response.json())
        .then(data => {
            streamEnabledSwitch.checked = data.enabled;
            streamEnabledSwitch.disabled = false;
        });
}

function toggleStream(checkbox) {
    streamEnabledSwitch.disabled = true;
    let body = {
        enabled: checkbox.target.checked
    };

    fetch('/api/admin/stream', {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(body),
            credentials: 'same-origin'
        })
        .then(response => {
            streamEnabledSwitch.disabled = false;

            if (!response.ok) {
                streamEnabledSwitch.checked = !streamEnabledSwitch.checked;
                showNotification('danger', 'Failed to update stream');
            }
        });
}

streamEnabledSwitch.onclick = toggleStream;

//#endregion


//#region Password Update

var userPwInput = document.getElementById('user-pw');
var adminPwInput = document.getElementById('admin-pw');
var updatePwButton = document.getElementById('update-pw');

// Enables or disables the button for submitting password updates
function setUpdateButtonEnabled() {
    if (userPwInput.value.length > 0 || adminPwInput.value.length > 0) {
        updatePwButton.disabled = false;
    } else {
        updatePwButton.disabled = true;
    }
}

// Invokes the admin API to change passwords
function updatePasswords() {
    let haveUserPw = userPwInput.value.length > 0;
    let haveAdminPw = adminPwInput.value.length > 0;

    // Shouldn't happen, but just in case
    if (!haveUserPw && !haveAdminPw) {
        console.warn('updatePasswords called unexpectedly');
        updatePwButton.disabled = true;
        return;
    }

    let body = {};
    if (haveUserPw) {
        body.userPw = userPwInput.value;
    }
    if (haveAdminPw) {
        body.adminPw = adminPwInput.value;
    }

    let confirmText = 'Are you sure you want to change this password?';
    if (haveUserPw && haveAdminPw) {
        confirmText = 'Are you sure you want to change these passwords?';
    }
    if (!confirm(confirmText)) {
        return;
    }

    updatePwButton.disabled = true;
    updatePwButton.classList.add('is-loading');

    fetch('/api/admin/passwords', {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(body),
            credentials: 'same-origin'
        })
        .then(response => {
            userPwInput.value = '';
            adminPwInput.value = '';
            updatePwButton.classList.remove('is-loading');

            if (!response.ok) {
                showNotification('danger', 'Failed to update passwords.');
            }
        });
}

userPwInput.oninput = setUpdateButtonEnabled;
adminPwInput.oninput = setUpdateButtonEnabled;
updatePwButton.onclick = updatePasswords;

//#endregion


//#region Session Reset

var sessionResetButton = document.getElementById('session-reset');
var resetModal = document.getElementById('reset-modal');

function sessionReset() {
    if (!confirm('All currently logged-in users (including you) will be logged out. Are you sure you want to do this?')) {
        return;
    }

    resetModal.classList.add('is-active');

    fetch('/api/admin/sessions', {
            method: 'DELETE',
            credentials: 'same-origin'
        })
        .then(handleSessionResetResponse);
}

function handleSessionResetResponse(response) {
    if (!response.ok) {
        resetModal.classList.remove('is-active');
        showNotification('danger', 'Failed to reset sessions.');
        return;
    }

    // Give the server time to restart, then reload
    new Promise(r => setTimeout(r, 5000))
        .then(() => {
            window.location.replace(window.location.href);
        })
}

sessionResetButton.onclick = sessionReset;

//#endregion


window.onload = function() {
    loadStreamState();
};
