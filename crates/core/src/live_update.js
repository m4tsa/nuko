const socket = new WebSocket(`ws://${window.location.host}/websocket`);

let last_revision = null;

function update() {
    fetch(location.toString())
        .then(response => response.text())
        .then(data => {
            document.open();
            document.write(data);
            document.close();
        }).catch(() => {
            window.location.reload();
        });
}

socket.addEventListener("message", function (event) {
    const revision = parseInt(event.data);

    if (Number.isInteger(last_revision) && last_revision != revision)
        update();

    last_revision = revision;
});
