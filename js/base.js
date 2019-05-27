// Site-wide behavior used by the base template


//#region Bulma Navbar Burger

var burgers = document.getElementsByClassName('navbar-burger');
var menus = document.getElementsByClassName('navbar-menu');

Array.from(burgers).forEach(burger => burger.onclick = toggleMenu);

function toggleMenu() {
    Array.from(burgers).forEach(burger => burger.classList.toggle('is-active'));
    Array.from(menus).forEach(menu => menu.classList.toggle('is-active'));
}

//#endregion
