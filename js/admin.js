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

            // TODO: check for and report success
            console.log(response)
        });
}


// -------------------------------------------------------------------------------------------------
// Password Update
// -------------------------------------------------------------------------------------------------

var userPwInput = document.getElementById('user-pw');
var adminPwInput = document.getElementById('admin-pw');
var updatePwButton = document.getElementById('update-pw')

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

            // TODO: check for and report success
            console.log(response)
        });
}

userPwInput.oninput = setUpdateButtonEnabled;
adminPwInput.oninput = setUpdateButtonEnabled;
updatePwButton.onclick = updatePasswords;


// -------------------------------------------------------------------------------------------------
// Session Reset
// -------------------------------------------------------------------------------------------------

var sessionResetButton = document.getElementById('session-reset')

function sessionReset() {
    if (!confirm('All currently logged-in users (including you) will be logged out. Are you sure you want to do this?')) {
        return;
    }

    fetch('/api/admin/sessions', {
            method: 'DELETE',
            credentials: 'same-origin'
        })
        .then(response => {

            // TODO: check for success, then reload page
            console.log(response)
        })
}

sessionResetButton.onclick = sessionReset;
