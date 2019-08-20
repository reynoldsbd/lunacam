class MessageBanner extends HTMLElement {

    //#region Web Component

    constructor() {

        super();

        this.body = document.getElementById('message-banner-template')
            .content
            .cloneNode(true);
        this.bodyAppended = false;

        // Bind template elements to properties of this object
        let elements = {
            deleteButton: 'delete-button',
            message: 'message',
            notification: 'notification',
        };
        Object.keys(elements).forEach(propertyName => {
            this[propertyName] = this.body.getElementById(elements[propertyName]);
            this[propertyName].removeAttribute('id');
        });

        // Bind event handlers
        this.deleteButton.onclick = e => this.onDeleteButtonClicked(e);
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

    //#endregion

    //#region Event Handlers

    onDeleteButtonClicked(_) {

        this.parentNode.removeChild(this);
    }

    //#endregion

    setMessage(message) {

        this.message.innerText = message;
    }

    setLevel(level) {

        this.notification.classList.add('is-' + level);
    }
}

customElements.define('message-banner', MessageBanner);


var messageArea = document.getElementById('message-area');

function showMessage(message, level) {

    if (messageArea == null) {
        console.warn('message area not found');
        return;
    }

    let messageBanner = document.createElement('message-banner');
    messageBanner.setMessage(message);

    switch (level) {
        case 'success':
            messageBanner.setLevel('primary');
            break;
        case 'warning':
            messageBanner.setLevel('warning');
            break;
        case 'error':
            messageBanner.setLevel('danger');
            break;
        default:
            console.warn('unknown message level \"' + level + '\"');
    }

    messageArea.appendChild(messageBanner);
}
