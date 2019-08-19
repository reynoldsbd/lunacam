const DUMMY_KEY = '......';

class CamEntry extends HTMLElement {

    //#region Web Component

    constructor() {

        super();

        this.body = document.getElementById('cam-entry-template')
            .content
            .cloneNode(true);
        this.bodyAppended = false;

        // Bind template elements to properties of this object
        let elements = {
            cancelButton: 'cancel-button',
            deleteButton: 'delete-button',
            dropdownIndicator: 'dropdown-indicator',
            enabledSwitch: 'cam-enabled',
            enabledSwitchLabel: 'cam-enabled-label',
            formWrapper: 'form-wrapper',
            header: 'header',
            hostnameField: 'cam-hostname-field',
            keyField: 'cam-key-field',
            nameLabel: 'cam-name-label',
            nameField: 'cam-name-field',
            orientationSelect: 'cam-orientation-select',
            saveButton: 'save-button',
        };
        Object.keys(elements).forEach(propertyName => {
            this[propertyName] = this.body.getElementById(elements[propertyName]);
            this[propertyName].removeAttribute('id');
        });

        // Bind event handlers
        this.cancelButton.onclick = e => this.onCancelButtonClicked(e);
        this.deleteButton.onclick = e => this.onDeleteButtonClicked(e);
        this.enabledSwitch.onclick = e => this.onEnabledSwitchClicked(e);
        this.header.onclick = e => this.onHeaderClicked(e);
        this.keyField.onfocus = e => this.onKeyFieldFocused(e);
        this.saveButton.onclick = e => this.onSaveButtonClicked(e);

        // Other initialization
        this.keyField.value = DUMMY_KEY;
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
            'cam-enabled',
            'cam-hostname',
            'cam-id',
            'cam-name',
            'cam-orientation',
        ];
    }

    attributeChangedCallback(name, _, newValue) {
        switch (name) {
            case 'cam-enabled':
                this.enabledSwitch.checked = (newValue == 'true');
                break;
            case 'cam-hostname':
                this.hostnameField.value = newValue;
                break;
            case 'cam-id':
                this.header.hidden = false;
                this.showDeleteButton();
                let switchId = newValue + '-enabled';
                this.enabledSwitch.setAttribute('id', switchId);
                this.enabledSwitchLabel.setAttribute('for', switchId);
                break;
            case 'cam-name':
                this.nameField.value = newValue;
                this.nameLabel.innerText = newValue;
                break;
            case 'cam-orientation':
                this.orientationSelect.value = newValue;
                break;
        }
    }

    //#endregion

    showMessage(message, level) {

        // TODO: needs to be user-visible

        switch (level) {
            case 'success':
                console.log('Success: ' + message);
                break;
            case 'error':
                console.error('Error: ' + message);
                break;
            default:
                console.warn('Unknown log level \"' + level + '\"');
                console.log(level + ': ' + message);
        }
    }

    reload(camera) {

        this.setAttribute('cam-enabled', camera.enabled);
        this.setAttribute('cam-hostname', camera.hostname);
        this.setAttribute('cam-id', camera.id);
        this.setAttribute('cam-name', camera.friendlyName);
        this.setAttribute('cam-orientation', camera.orientation);
        this.keyField.value = DUMMY_KEY;
    }

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

    uploadCamera(camera) {

        // Only allow one in-flight submission at a time
        if (this.activeSubmission) {
            return;
        }

        this.activeSubmission = true;
        this.saveButton.disabled = true;
        this.saveButton.classList.add('is-loading');
        this.cancelButton.disabled = true;
        this.enabledSwitch.disabled = true;

        let url = '/api/cameras';
        let init = {
            headers: {
                'Content-Type': 'application/json'
            },
            credentials: 'same-origin'
        };

        if (this.hasAttribute('cam-id')) {
            url += '/' + this.getAttribute('cam-id');
            init.method = 'PATCH';
        } else {
            init.method = 'PUT';
        }
        init.body = JSON.stringify(camera);

        fetch(url, init)
            .then(r => this.handleUploadResponse(r));
    }

    handleUploadResponse(response) {

        this.activeSubmission = false;
        this.saveButton.disabled = false;
        this.saveButton.classList.remove('is-loading');
        this.cancelButton.disabled = false;
        this.enabledSwitch.disabled = false;

        let jsonPromise = response.json();

        if (response.ok) {
            jsonPromise.then(c => {
                this.reload(c);
                this.showMessage('Camera changes were saved successfully', 'success');
            });

        } else {
            jsonPromise.then(e => this.showMessage(e.message, 'error'));
        }
    }

    deleteCamera() {

        let url = '/api/cameras/' + this.getAttribute('cam-id');
        let init = {
            method: 'DELETE',
            credentials: 'same-origin',
        };

        fetch(url, init)
            .then(r => this.handleDeleteResponse(r));
    }

    handleDeleteResponse(response) {

        if (response.ok) {
            this.showMessage('Camera successfully deleted', 'success');
            this.parentElement.removeChild(this);

        } else {
            response.json()
                .then(e => this.showMessage(e.message, 'error'));
        }
    }

    //#endregion

    //#region Event Handlers

    onCancelButtonClicked(_) {

        // If this entry is unsaved, delete it when the cancel button is pressed
        if (!this.hasAttribute('cam-id')) {
            this.parentElement.removeChild(this);

        // Otherwise, reset form contents to initial values
        } else {
            this.enabledSwitch.checked = (this.getAttribute('cam-enabled') == 'true');
            this.hostnameField.value = this.getAttribute('cam-hostname');
            this.nameField.value = this.getAttribute('cam-name');
            this.orientationSelect.value = this.getAttribute('cam-orientation');
            this.keyField.value = DUMMY_KEY;
        }
    }

    onDeleteButtonClicked(_) {

        if (confirm('Are you sure you want to delete this camera?')) {
            this.deleteCamera();
        }
    }

    onEnabledSwitchClicked(_) {

        let camera = {
            enabled: this.enabledSwitch.checked,
        };

        this.uploadCamera(camera);
    }

    onHeaderClicked(_) {

        if (this.formWrapper.hidden) {
            this.showForm();
        } else {
            this.hideForm();
        }
    }

    onKeyFieldFocused(_) {

        if (this.keyField.value == DUMMY_KEY) {
            this.keyField.select();
        }
    }

    onSaveButtonClicked(_) {

        let camera = {
            hostname: this.hostnameField.value,
            friendlyName: this.nameField.value,
            orientation: this.orientationSelect.value,
        };
        if (!this.hasAttribute('cam-id') || this.keyField.value != DUMMY_KEY) {
            camera.deviceKey = this.keyField.value;
        }

        this.uploadCamera(camera);
    }

    //#endregion
}

customElements.define('cam-entry', CamEntry);


var addCameraButton = document.getElementById('add-button');
var cameraList = document.getElementById('cam-list');

function addCamera() {

    let newCamEntry = document.createElement('cam-entry');
    newCamEntry.hideDeleteButton();
    newCamEntry.header.hidden = true;
    newCamEntry.showForm();
    newCamEntry.keyField.value = '';
    cameraList.appendChild(newCamEntry);
}

addCameraButton.onclick = addCamera;
