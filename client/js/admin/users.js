class UserEntry extends HTMLElement {

    //#region Web Component

    constructor() {

        super();

        this.body = document.getElementById('user-entry-template')
            .content
            .cloneNode(true);
        this.bodyAppended = false;

        // Bind template elemets to properties of this object
        let elements = {
            cancelButton: 'cancel-button',
            deleteButton: 'delete-button',
            dropdownIndicator: 'dropdown-indicator',
            formWrapper: 'form-wrapper',
            header: 'header',
            passwordConfirmField: 'password-cfrm-field',
            passwordField: 'password-field',
            saveButton: 'save-button',
            usernameLabel: 'username-label',
            usernameField: 'username-field'
        };
        Object.keys(elements).forEach(propertyName => {
            this[propertyName] = this.body.getElementById(elements[propertyName]);
            this[propertyName].removeAttribute('id');
        });

        // Bind event handlers
        this.cancelButton.onclick = e => this.onCancelButtonClicked(e);
        this.deleteButton.onclick = e => this.onDeleteButtonClicked(e);
        this.header.onclick = e => this.onHeaderClicked(e);
        this.saveButton.onclick = e => this.onSaveButtonClicked(e);
    }

    connectedCallback() {

        // Since this element doesn't use a shadow DOM, we are prohibited from calling appendChild
        // in the constructor. Instead, this step is deferred to connectedCallback.
        // https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element-conformance
        if (!this.bodyAppended) {
            this.appendChild(this.body);
            this.bodyAppended = true;
        }
    }

    static get observedAttributes() {
        return [
            'username',
            'user-id',
        ];
    }

    attributeChangedCallback(name, _, newValue) {
        switch (name) {
            case 'username':
                this.usernameLabel.innerText = newValue;
                this.usernameField.value = newValue;
                break;
            case 'user-id':
                this.header.hidden = false;
                this.showDeleteButton();
                break;
        }
    }

    //#endregion

    //#region Form Display

    showForm() {
        this.dropdownIndicator.classList.remove('fa-chevron-down');
        this.dropdownIndicator.classList.add('fa-chevron-up');
        this.formWrapper.hidden = false;
    }

    hideForm() {
        this.dropdownIndicator.classList.remove('fa-chevron-up');
        this.dropdownIndicator.classList.add('fa-chevron-down');
        this.formWrapper.hidden = true;
    }

    showDeleteButton() {
        this.deleteButton.classList.add('button')
        this.deleteButton.hidden = false;
    }

    hideDeleteButton() {
        this.deleteButton.classList.remove('button');
        this.deleteButton.hidden = true;
    }

    //#endregion

    //#region API Interaction

    uploadUser(user) {

        // Only allow one in-flight submission at a time
        if (this.activeSubmission) {
            return;
        }

        this.activeSubmission = true;
        this.saveButton.disabled = true;
        this.saveButton.classList.add('is-loading');
        this.cancelButton.disabled = true;

        let url = '/api/users';
        let init = {
            headers: {
                'Content-Type': 'application/json'
            },
            credentials: 'same-origin',
        };

        if (this.hasAttribute('user-id')) {
             url += '/' + this.getAttribute('user-id');
             init.method = 'PATCH';
        } else {
            init.method = 'PUT';
        }
        init.body = JSON.stringify(user);

        fetch(url, init)
            .then(r => this.handleUploadResponse(r));
    }

    handleUploadResponse(response) {

        this.activeSubmission = false;
        this.saveButton.disabled = false;
        this.saveButton.classList.remove('is-loading');
        this.cancelButton.disabled = false;

        let jsonPromise = response.json();

        if (response.ok) {
            jsonPromise.then(u => {
                this.reload(u);
                showMessage('User changes were saved successfully', 'success');
            });
        } else {
            jsonPromise.then(e => showMessage(e.message, 'error'));
        }
    }

    deleteUser() {

        let url = '/api/users/' + this.getAttribute('user-id');
        let init = {
            method: 'DELETE',
            credentials: 'same-origin',
        };

        fetch(url, init)
            .then(r => this.handleDeleteResponse(r));
    }

    handleDeleteResponse(response) {

        if (response.ok) {
            showMessage('User successfully deleted', 'success');
            this.parentElement.removeChild(this);

        } else {
            repsonse.json()
                .then(e => showMessage(e.message, 'error'));
        }
    }

    //#endregion

    //#region Event Handlers

    onCancelButtonClicked(_) {

        // If this entry is unsaved, delete it when the cancel button is pressed
        if (!this.hasAttribute('user-id')) {
            this.parentElement.removeChild(this);

        // Otherwise, reset form contents to initial values
        } else {
            this.passwordConfirmField.value = '';
            this.passwordField.value = '';
            this.usernameField.value = this.getAttribute('username');
        }
    }

    onDeleteButtonClicked(_) {

        if (confirm('Are you sure you want to delete this user?')) {
            this.deleteUser();
        }
    }

    onHeaderClicked(_) {

        if (this.formWrapper.hidden) {
            this.showForm();
        } else {
            this.hideForm();
        }
    }

    onSaveButtonClicked(_) {

        let user = {
            username: this.usernameField.value,
        };

        if (this.passwordField.value) {
            if (this.passwordField.value != this.passwordConfirmField.value) {
                showMessage('Passwords do not match', 'error');
                return;
            }

            user.password = this.passwordField.value;
        }

        this.uploadUser(user);
    }

    //#endregion
}

customElements.define('user-entry', UserEntry);


var addUserButton = document.getElementById('add-button');
var userList = document.getElementById('user-list');

function addUser() {

    let newUserEntry = document.createElement('user-entry');
    newUserEntry.hideDeleteButton();
    newUserEntry.header.hidden = true;
    newUserEntry.showForm();
    userList.appendChild(newUserEntry);
}

addUserButton.onclick = addUser;
