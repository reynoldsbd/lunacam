class CamEntry extends HTMLElement {

    constructor() {

        super();

        this.body = document.getElementById('cam-entry-template')
            .content
            .cloneNode(true);
        this.bodyAppended = false;

        // TODO: this.nameLabel
        this.name = this.body.getElementById('cam-name');
        this.name.removeAttribute('id');

        this.enabledSwitch = this.body.getElementById('cam-enabled');
        this.enabledSwitch.removeAttribute('id');
        this.enabledSwitchLabel = this.body.getElementById('cam-enabled-label');
        this.enabledSwitchLabel.removeAttribute('id');

        // TODO: callbacks
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
            'cam-name',
            'cam-id',
        ];
    }

    attributeChangedCallback(name, _, newValue) {
        switch (name) {
            case 'cam-name':
                this.name.innerText = newValue;
                break;
            case 'cam-id':
                let switchId = newValue + '-enabled';
                this.enabledSwitch.setAttribute('id', switchId);
                this.enabledSwitchLabel.setAttribute('for', switchId);
                break;
        }
    }
}

customElements.define('cam-entry', CamEntry);
