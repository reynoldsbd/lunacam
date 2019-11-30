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
            addressField: 'cam-address-field',
            addressWrapper: 'cam-address-wrapper',
            cancelButton: 'cancel-button',
            deleteButton: 'delete-button',
            dropdownIndicator: 'dropdown-indicator',
            enabledSwitch: 'cam-enabled',
            enabledSwitchLabel: 'cam-enabled-label',
            formWrapper: 'form-wrapper',
            header: 'header',
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
            'cam-address',
            'cam-enabled',
            'cam-id',
            'cam-local',
            'cam-name',
            'cam-orientation',
        ];
    }

    attributeChangedCallback(name, _, newValue) {
        switch (name) {
            case 'cam-address':
                this.addressField.value = newValue;
                break;
            case 'cam-enabled':
                this.enabledSwitch.checked = (newValue == 'true');
                break;
            case 'cam-id':
                this.header.hidden = false;
                if (!this.isLocal) {
                    this.showDeleteButton();
                }
                let switchId = newValue + '-enabled';
                this.enabledSwitch.setAttribute('id', switchId);
                this.enabledSwitchLabel.setAttribute('for', switchId);
                break;
            case 'cam-local':
                if (newValue == 'true') {
                    this.isLocal = true;
                    this.addressWrapper.hidden = true;
                    this.hideDeleteButton();
                }
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

    reload(camera) {

        this.setAttribute('cam-address', camera.address);
        this.setAttribute('cam-enabled', camera.enabled);
        this.setAttribute('cam-id', camera.id);
        this.setAttribute('cam-name', camera.name);
        this.setAttribute('cam-orientation', camera.orientation);
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

    uploadCamera(camera, showSuccessMessage = true) {

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
            .then(r => this.handleUploadResponse(r, showSuccessMessage));
    }

    handleUploadResponse(response, showSuccessMessage) {

        this.activeSubmission = false;
        this.saveButton.disabled = false;
        this.saveButton.classList.remove('is-loading');
        this.cancelButton.disabled = false;
        this.enabledSwitch.disabled = false;

        let jsonPromise = response.json();

        if (response.ok) {
            jsonPromise.then(c => {
                this.reload(c);
                if (showSuccessMessage) {
                    showMessage('Camera changes were saved successfully', 'success');
                }
            });

        } else {
            jsonPromise.then(e => showMessage(e.message, 'error'));
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
            showMessage('Camera successfully deleted', 'success');
            this.parentElement.removeChild(this);

        } else {
            response.json()
                .then(e => showMessage(e.message, 'error'));
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
            this.addressField.value = this.getAttribute('cam-address');
            this.enabledSwitch.checked = (this.getAttribute('cam-enabled') == 'true');
            this.nameField.value = this.getAttribute('cam-name');
            this.orientationSelect.value = this.getAttribute('cam-orientation');
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

        this.uploadCamera(camera, false);
    }

    onHeaderClicked(_) {

        if (this.formWrapper.hidden) {
            this.showForm();
        } else {
            this.hideForm();
        }
    }

    onSaveButtonClicked(_) {

        let camera = {
            name: this.nameField.value,
            orientation: this.orientationSelect.value,
        };

        if (!this.isLocal) {
            camera.address = this.addressField.value;
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
    cameraList.appendChild(newCamEntry);
}

addCameraButton.onclick = addCamera;
