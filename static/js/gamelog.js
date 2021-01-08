document.getElementById("allevents").addEventListener('change', function () {
    if (this.value) {
        setVisible("#events > .log-chat-entry", true)
        setVisible("#events > .log-entry", true)
    }
})

document.getElementById("gameonly").addEventListener('change', function () {
    if (this.value) {
        setVisible("#events > .log-chat-entry", false)
        setVisible("#events > .log-entry", true)
    }
})

document.getElementById("chatonly").addEventListener('change', function () {
    if (this.value) {
        setVisible("#events > .log-chat-entry", true)
        setVisible("#events > .log-entry", false)
    }
})

function setVisible(selector, visible) {
    document.querySelectorAll(selector).forEach(e => e.setAttribute("style", `display: ${visible ? "inherit" : "none !important"}`))
}