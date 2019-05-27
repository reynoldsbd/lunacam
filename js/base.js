// Site-wide behavior used by the base template


//#region Bulma Navbar Burger

var burgers = document.getElementsByClassName('navbar-burger');
var menus = document.getElementsByClassName('navbar-menu');

function toggleMenu() {
    Array.from(burgers).forEach(burger => burger.classList.toggle('is-active'));
    Array.from(menus).forEach(menu => menu.classList.toggle('is-active'));
}

Array.from(burgers).forEach(burger => burger.onclick = toggleMenu);

//#endregion


//#region Notifications

var notificationArea = document.getElementById('notifications');
var initialNotifications = document.getElementsByClassName('notification');

function setDeleteHandlers(notification) {
    // https://bulma.io/documentation/elements/notification/
    let handler = () => notification.parentNode.removeChild(notification);
    let deleteButtons = notification.getElementsByClassName('delete');
    Array.from(deleteButtons).forEach(button => button.onclick = handler);
}

Array.from(initialNotifications).forEach(setDeleteHandlers);

function showNotification(notificationType, message) {
    // https://davidwalsh.name/convert-html-stings-dom-nodes
    let content =
        '<div class="notification is-' + notificationType + '">' +
            '<button class="delete"></button>' +
            message +
        '</div>';
    let notification = document.createRange()
        .createContextualFragment(content)
        .firstChild;

    setDeleteHandlers(notification);
    notificationArea.appendChild(notification);
}

//#endregion
