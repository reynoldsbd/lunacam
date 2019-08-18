class CamEntry extends HTMLElement {

    constructor() {

        super();

        this.body = document.getElementById('cam-entry-template')
            .content
            .cloneNode(true);
        this.bodyAppended = false;

        let elements = {
            dropdownIndicator: 'dropdown-indicator',
            enabledSwitch: 'cam-enabled',
            enabledSwitchLabel: 'cam-enabled-label',
            formWrapper: 'form-wrapper',
            header: 'header',
            hostnameField: 'cam-hostname-field',
            nameLabel: 'cam-name-label',
            nameField: 'cam-name-field',
            orientationSelect: 'cam-orientation-select'
        };
        Object.keys(elements).forEach(propertyName => {
            this[propertyName] = this.body.getElementById(elements[propertyName]);
            this[propertyName].removeAttribute('id');
        });

        this.formWrapper.hidden = true;
        this.formHidden = true;
        this.header.onclick = () => this.toggleForm();
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
}

customElements.define('cam-entry', CamEntry);
