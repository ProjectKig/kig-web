const locals = JSON.parse(localStorage.getItem("kig-theme"))
const stylesheet = document.getElementById("bootstrap-dark")
const toggler = document.getElementById("theme-toggler")

setDarkTheme(locals && locals.dark)

function toggleDarkTheme() {
    setDarkTheme(!darkTheme)
}

function setDarkTheme(dark) {
    darkTheme = dark
    stylesheet.setAttribute("href", dark ? "/game-static/css/bootstrap-dark.min.css" : "")
    document.body.setAttribute("data-theme", dark ? "dark" : "") // For other CSS files
    localStorage.setItem("kig-theme", JSON.stringify({ dark }))
    toggler.innerHTML = `Theme: ${dark ? "Dark" : "Light"}`
}