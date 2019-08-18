function debounce(fn, ms) {
    let timer;
    return function(...args) {
        clearTimeout(timer);
        timer = setTimeout(() => fn(...args), ms || 0);
    }
}


function isUndefinedOrWhiteSpace(str) {
    return !str || !str.trim();
}


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

        this.formWrapper.hidden = true;
        this.formHidden = true;
        this.header.onclick = () => this.toggleForm();

        let prepareSubmission = debounce(e => this.prepareSubmission(e), 500);
        this.enabledSwitch.onclick = prepareSubmission;
        this.hostnameField.oninput = prepareSubmission;
        this.nameField.oninput = prepareSubmission;
        this.orientationSelect.onchange = prepareSubmission;
        this.keyField.oninput = prepareSubmission;

        this.activeSubmission = false;
        this.saveButton.onclick = e => this.submitForm(e);
        this.cancelButton.onclick = e => this.clearForm(e);
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
        this.setAttribute('cam-name', camera.friendlyName);
        this.setAttribute('cam-orientation', camera.orientation);
        this.keyField.value = '......';
    }

    toggleForm() {
        if (this.formHidden) {
            this.dropdownIndicator.classList.remove('fa-chevron-down');
            this.dropdownIndicator.classList.add('fa-chevron-up');
            this.formWrapper.hidden = false;
            this.formHidden = false;
        } else {
            this.dropdownIndicator.classList.remove('fa-chevron-up');
            this.dropdownIndicator.classList.add('fa-chevron-down');
            this.formWrapper.hidden = true;
            this.formHidden = true;
        }
    }

    prepareSubmission(_event) {

        let submissionBody = {};
        let enableButtons = false;

        // Only include things that are different in the PATCH

        if (this.enabledSwitch.checked != (this.getAttribute('cam-enabled') == 'true')) {
            submissionBody.enabled = this.enabledSwitch.checked;
            enableButtons = true;
        }

        if (this.hostnameField.value != this.getAttribute('cam-hostname')) {
            submissionBody.hostname = this.hostnameField.value;
            enableButtons = true;
        }

        if (this.nameField.value != this.getAttribute('cam-name')) {
            submissionBody.friendlyName = this.nameField.value;
            enableButtons = true;
        }

        if (this.orientationSelect.value != this.getAttribute('cam-orientation')) {
            submissionBody.orientation = this.orientationSelect.value;
            enableButtons = true;
        }

        if (this.keyField.value != '......' && !isUndefinedOrWhiteSpace(this.keyField.value)) {
            submissionBody.deviceKey = this.keyField.value;
            enableButtons = true;
        }

        if (enableButtons) {
            this.submissionBody = submissionBody;
            this.saveButton.disabled = false;
            this.cancelButton.disabled = false;
        } else {
            delete this.submissionBody;
            this.saveButton.disabled = true;
            this.cancelButton.disabled = true;
        }
    }

    handleSubmissionResponse(response) {

        this.activeSubmission = false;
        this.saveButton.classList.remove('is-loading');

        let jsonPromise = response.json();

        if (response.ok) {

            jsonPromise.then(c => {
                this.reload(c);
                this.showMessage('Camera changes were saved successfully', 'success');
            });
            delete this.submissionBody;

        } else {

            jsonPromise.then(e => this.showMessage(e.message, 'error'));
            this.saveButton.disabled = false;
            this.cancelButton.disabled = false;
        }
    }

    submitForm(_event) {

        // Only allow one in-flight PATCH at a time
        if (this.activeSubmission) {
            return;
        }

        this.activeSubmission = true;
        this.saveButton.disabled = true;
        this.saveButton.classList.add('is-loading');
        this.cancelButton.disabled = true;

        let init = {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(this.submissionBody),
            credentials: 'same-origin'
        };

        fetch('/api/cameras/' + this.getAttribute('cam-id'), init)
            .then(r => this.handleSubmissionResponse(r));
    }

    clearForm(_event) {

        this.enabledSwitch.checked = (this.getAttribute('cam-enabled') == 'true');
        this.hostnameField.value = this.getAttribute('cam-hostname');
        this.nameField.value = this.getAttribute('cam-name');
        this.orientationSelect.value = this.getAttribute('cam-orientation');
        this.keyField.value = '......';

        delete this.submissionBody;
        this.saveButton.disabled = true;
        this.cancelButton.disabled = true;
    }
}

customElements.define('cam-entry', CamEntry);
