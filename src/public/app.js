(async function() {
    document.querySelector('.host').addEventListener('click', () => {
        (async function() {
            const response = await fetch("/rooms", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    name: "dac"
                }),
            });
            let id = await response.text();
            console.log(id);

            const hostSocket = new WebSocket(`ws://localhost:8080/rooms/${id}/host`);
            hostSocket.addEventListener("message", (message) => {
                console.log(message.data);
            });
        })();
    });

    document.querySelector('.join').addEventListener('click', () => {
        (async function() {
            const response = await fetch('/rooms/id?name=dac', {
                method: 'GET',
            });
            let id = await response.text();
            console.log(id);

            const participateSocket = new WebSocket(`ws://localhost:8080/rooms/${id}/participate?name=Scotow`);
            participateSocket.addEventListener("message", (message) => {
                console.log(message.data);
            });

            document.getElementById('buzzer').addEventListener('click', () => {
                participateSocket.send('buzz');
            });
        })();
    });

})().then(out => {
    console.log(out);
});