const DUMMY_KEY = '......';

class CamEntry extends HTMLElement {

    //#region Web Component

    constructor() {

        super();

        this.body = document.getElementById('cam-entry-template')
            .content
            .cloneNode(true);
        this.bodyAppended = false;

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

        this.header.onclick = () => this.toggleForm();
        this.header.hidden = true;

        this.activeSubmission = false;
        this.saveButton.onclick = e => this.submitForm(e);
        this.cancelButton.onclick = e => this.cancelEntry(e);

        this.deleteButton.onclick = e => this.deleteEntry(e);
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
                this.hideForm();
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

    toggleForm() {
        if (this.formWrapper.hidden) {
            this.showForm();
        } else {
            this.hideForm();
        }
    }

    //#endregion

    submitForm(_event) {

        // Only allow one in-flight submission at a time
        if (this.activeSubmission) {
            return;
        }

        this.activeSubmission = true;
        this.saveButton.disabled = true;
        this.saveButton.classList.add('is-loading');
        this.cancelButton.disabled = true;

        let url = '/api/cameras';
        let body = {
            enabled: this.enabledSwitch.checked,
            hostname: this.hostnameField.value,
            friendlyName: this.nameField.value,
            orientation: this.orientationSelect.value,
        };
        let init = {
            headers: {
                'Content-Type': 'application/json'
            },
            credentials: 'same-origin'
        };

        if (this.hasAttribute('cam-id')) {
            url += '/' + this.getAttribute('cam-id');
            body.deviceKey = this.keyField.value;
            init.method = 'PATCH';
        } else {
            init.method = 'PUT';
            if (this.keyField.value != DUMMY_KEY) {
                body.deviceKey = this.keyField.value;
            }
        }
        init.body = JSON.stringify(body);

        fetch(url, init)
            .then(r => this.handleSubmissionResponse(r));
    }

    handleSubmissionResponse(response) {

        this.activeSubmission = false;
        this.saveButton.disabled = false;
        this.saveButton.classList.remove('is-loading');
        this.cancelButton.disabled = false;

        let jsonPromise = response.json();

        if (response.ok) {
            jsonPromise.then(c => {
                this.reload(c);
                this.showForm();
                this.showMessage('Camera changes were saved successfully', 'success');
            });

        } else {
            jsonPromise.then(e => this.showMessage(e.message, 'error'));
        }
    }

    cancelEntry(_event) {

        // If this entry is unsaved, delete it when the cancel button is pressed
        if (!this.hasAttribute('cam-id')) {
            this.parentElement.removeChild(this);

        // Otherwise, this entry corresponds to an existing camera. Reset form contents to initial
        // values and disable controls.
        } else {
            this.enabledSwitch.checked = (this.getAttribute('cam-enabled') == 'true');
            this.hostnameField.value = this.getAttribute('cam-hostname');
            this.nameField.value = this.getAttribute('cam-name');
            this.orientationSelect.value = this.getAttribute('cam-orientation');
            this.keyField.value = DUMMY_KEY;
        }
    }

    deleteEntry(_event) {

        if (!confirm('Are you sure you want to delete this camera?')) {
            return;
        }

        fetch('/api/cameras/' + this.getAttribute('cam-id'), {
                method: 'DELETE',
                credentials: 'same-origin'
            })
            .then(r => this.handleDeletionResponse(r));
    }

    handleDeletionResponse(response) {

        if (response.ok) {
            this.showMessage('Camera successfully deleted', 'success');
            this.parentElement.removeChild(this);

        } else {
            response.json()
                .then(e => this.showMessage(e.message, 'error'));
        }
    }
}

customElements.define('cam-entry', CamEntry);


var addCameraButton = document.getElementById('add-button');
var cameraList = document.getElementById('cam-list');

function addCamera() {

    let newCamEntry = new CamEntry();
    cameraList.appendChild(newCamEntry);
}

addCameraButton.onclick = addCamera;
